use crate::cmd::commands::BindDset;
use crate::cmd::commands::BindPipeline;
use crate::cmd::commands::Draw;
use crate::cmd::commands::DrawKind;
use crate::cmd::commands::DrawManaged;
use crate::cmd::stream::*;
use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FreeBlock {
  pub index: usize,
  pub count: usize,
}
impl PartialOrd for FreeBlock {
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    Some(self.count.cmp(&other.count))
  }
}
impl Ord for FreeBlock {
  fn cmp(&self, other: &Self) -> std::cmp::Ordering {
    self.count.cmp(&other.count)
  }
}

pub struct BlockAlloc<T: Clone + Copy> {
  data: Vec<T>,
  flag: Vec<bool>,
  free: BTreeSet<FreeBlock>,
}

impl<T: Clone + Copy> Default for BlockAlloc<T> {
  fn default() -> Self {
    Self {
      data: Default::default(),
      flag: Default::default(),
      free: Default::default(),
    }
  }
}

impl<T: Clone + Copy> BlockAlloc<T> {
  pub fn push(&mut self, elems: &[T]) -> (usize, bool) {
    match self.free.iter().find(|b| b.count >= elems.len()).cloned() {
      Some(b) => {
        for (dst, src) in self.data[b.index..b.index + b.count].iter_mut().zip(elems.iter()) {
          *dst = *src;
        }
        for f in self.flag[b.index..b.index + b.count].iter_mut() {
          *f = true;
        }
        self.free.remove(&b);
        if b.count > elems.len() {
          self.free.insert(FreeBlock {
            index: b.index + elems.len(),
            count: b.count - elems.len(),
          });
        }
        (b.index, false)
      }
      None => {
        let capacity = self.data.capacity();
        let pos = self.data.len();
        for e in elems.iter() {
          self.data.push(*e);
          self.flag.push(true);
        }
        (pos, capacity != self.data.capacity())
      }
    }
  }

  pub fn free(&mut self, index: usize, count: usize) {
    for f in self.flag[index..index + count].iter_mut() {
      *f = false;
    }

    // find overlapping free blocks
    let overlap = self
      .free
      .iter()
      .filter(|b| index == b.index + b.count || b.index == index + count)
      .cloned()
      .collect::<Vec<_>>();

    // begin and size of combined free block
    let index = overlap.iter().map(|b| b.index).min().unwrap_or(0);
    let count = overlap.iter().fold(count, |acc, b| acc + b.count);

    for b in overlap {
      self.free.remove(&b);
    }
    self.free.insert(FreeBlock { index, count });
  }

  pub fn clear(&mut self) {
    self.data.clear();
    self.flag.clear();
    self.free.clear();
  }

  pub fn contains(&self, index: usize) -> bool {
    self.flag[index]
  }

  pub fn iter(&self) -> impl Iterator<Item = &T> + '_ {
    self.data.iter().zip(self.flag.iter()).filter(|(_, f)| **f == true).map(|(e, _)| e)
  }
  pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> + '_ {
    self
      .data
      .iter_mut()
      .zip(self.flag.iter())
      .filter(|(_, f)| **f == true)
      .map(|(e, _)| e)
  }
}

impl<T: Clone + Copy> std::ops::Index<usize> for BlockAlloc<T> {
  type Output = T;
  fn index(&self, i: usize) -> &Self::Output {
    &self.data[i]
  }
}
impl<T: Clone + Copy> std::ops::IndexMut<usize> for BlockAlloc<T> {
  fn index_mut(&mut self, i: usize) -> &mut Self::Output {
    &mut self.data[i]
  }
}
impl<T: Clone + Copy> std::ops::Index<std::ops::Range<usize>> for BlockAlloc<T> {
  type Output = [T];
  fn index(&self, r: std::ops::Range<usize>) -> &Self::Output {
    &self.data[r]
  }
}
impl<T: Clone + Copy> std::ops::IndexMut<std::ops::Range<usize>> for BlockAlloc<T> {
  fn index_mut(&mut self, r: std::ops::Range<usize>) -> &mut Self::Output {
    &mut self.data[r]
  }
}

/// Id type to uniquely identify a drawable mesh
///
/// We define a separate type for this so that we get more type checking from the compiler.
/// Conversion to and from `usize` is still available with `into()` and `from()`, however we have to conscientously do this.
/// This way it is less likely to accidentally use an `usize` as a MeshId that is in fact not.
///
/// We can also forbid certain operations on ids (eg. mul, div, bit logic) because that doesn't make sense for ids.
/// However we allow addition with ***vanilla*** usize and getting the difference between two MeshIds.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MeshId {
  id: usize,
}

impl MeshId {
  /// We reserve `usize::max_value` as an invalid id that must not be used.
  pub fn invalid() -> Self {
    Self { id: usize::max_value() }
  }
}

impl Into<usize> for MeshId {
  fn into(self) -> usize {
    self.id
  }
}
impl From<usize> for MeshId {
  fn from(id: usize) -> Self {
    Self { id }
  }
}

impl std::ops::Add<usize> for MeshId {
  type Output = Self;
  fn add(self, rhs: usize) -> Self {
    (self.id + rhs).into()
  }
}
impl std::ops::Sub for MeshId {
  type Output = isize;
  fn sub(self, rhs: Self) -> isize {
    self.id as isize - rhs.id as isize
  }
}

#[derive(Debug, Clone, Copy)]
struct DrawMesh {
  toggle: bool,
  pipe: usize,
  dset: (usize, usize),
  buffers: (usize, usize),
  draw: Draw,
}
/// Compiled info for a single draw call
#[derive(Debug)]
pub struct DrawMeshRef<'a> {
  pub pipe: &'a BindPipeline,
  pub dset: &'a [BindDset],
  pub buffers: &'a [vk::Buffer],
  pub offsets: &'a [vk::DeviceSize],
  pub draw: &'a Draw,
}
impl<'a> StreamPush for DrawMeshRef<'a> {
  fn enqueue(&self, mut cs: CmdBuffer) -> CmdBuffer {
    cs = cs.push(self.pipe);
    for ds in self.dset.iter() {
      cs = cs.push(ds);
    }
    cs.push(self.draw)
  }
}

/// Manager for DrawCommands
///
/// The DrawPass records information that is needed to execute a single draw call in vulkan. We call the compiled information `Mesh` and can refer to it through a [MeshId](struct.MeshId.html)
/// It stores pipeline, descriptor sets, vertex buffers with offsets, and the [Draw](../cmd/commands/struct.Draw.html) call itself in an struct of array manner.
/// Descriptor sets and vertex buffers that belong to the same draw call are stored packed together.
///
/// The DrawPass can be enqued into a [command buffer](../cmd/struct.CmdBuffer.html), which will iterate over all recorded draw calles and execute them accordingly.
/// We can also define a draw order with [iter](struct.DrawPass.html#method.iter)
pub struct DrawPass {
  pipes: BlockAlloc<BindPipeline>,
  dsets: BlockAlloc<BindDset>,
  buffers: BlockAlloc<vk::Buffer>,
  offsets: BlockAlloc<vk::DeviceSize>,

  meshes: BlockAlloc<DrawMesh>,
}

impl DrawPass {
  /// Creates a new (empty) DrawPass
  pub fn new() -> Self {
    Self {
      pipes: Default::default(),
      dsets: Default::default(),
      buffers: Default::default(),
      offsets: Default::default(),

      meshes: Default::default(),
    }
  }

  fn update_bindvbs(&mut self) {
    for m in self.meshes.iter_mut() {
      let beg = m.buffers.0;
      let end = m.buffers.0 + m.buffers.1;
      m.draw.vbs = (&self.buffers[beg..end], &self.offsets[beg..end]).into();
    }
  }

  /// Allocate a new mesh
  ///
  /// Adds a mesh. Requires a valid [BindPipeline](../cmd/commands/struct.BindPipeline.html) and [DrawManaged](../cmd/commands/struct.DrawManaged.html) command.
  /// We optionally can use one or more [BindDset](../cmd/commands/struct.BindDset.html) commands.
  ///
  /// This function takes a [DrawManaged](../cmd/commands/struct.DrawManaged.html) command and inserts the individually allocated vertex buffers into the DrawPasses to improve cache coherence.
  ///
  /// # Returns
  /// The [MeshId](struct.MeshId.html) associated with the new mesh
  pub fn new_mesh(&mut self, pipe: BindPipeline, dsets: &[BindDset], draw: DrawManaged) -> MeshId {
    // copy into draw pass allocators
    let (pipe, _) = self.pipes.push(&[pipe]);
    let (dset, _) = self.dsets.push(dsets);
    let (buffers, realloc_vbs) = self.buffers.push(&draw.vbs.buffers);
    let (offsets, _) = self.offsets.push(&draw.vbs.offsets);
    debug_assert!(buffers == offsets, "inconsistent buffers and offsets index");

    // create the DrawMesh with 'unmanaged' BindVertexBuffers
    let beg = buffers;
    let end = buffers + draw.vbs.buffers.len();
    let (mesh, _) = self.meshes.push(&[DrawMesh {
      toggle: true,
      pipe,
      dset: (dset, dsets.len()),
      buffers: (buffers, draw.vbs.buffers.len()),
      draw: Draw::new((&self.buffers[beg..end], &self.offsets[beg..end]).into(), draw.draw),
    }]);

    // If buffers have been reallocated we need to make sure the BindVertexBuffers of all meshes are updated
    if realloc_vbs {
      self.update_bindvbs();
    }

    MeshId::from(mesh)
  }

  /// Updates the mesh
  ///
  /// Updating the mesh may NOT change the number of descriptor sets or vertex buffers!
  pub fn update_mesh(
    &mut self,
    mesh: MeshId,
    pipe: Option<BindPipeline>,
    dsets: &[Option<BindDset>],
    buffers: &[Option<vk::Buffer>],
    draw: Option<DrawKind>,
  ) {
    struct Mut<'a> {
      pipe: &'a mut BindPipeline,
      dset: &'a mut [BindDset],
      buffers: &'a mut [vk::Buffer],
      draw: &'a mut Draw,
    };
    let m: usize = mesh.into();
    let m = &mut self.meshes[m];
    let mesh = Mut {
      pipe: &mut self.pipes[m.pipe],
      dset: &mut self.dsets[m.dset.0..m.dset.0 + m.dset.1],
      buffers: &mut self.buffers[m.buffers.0..m.buffers.0 + m.buffers.1],
      draw: &mut m.draw,
    };

    //let mesh = self.get_mut(mesh);
    if let Some(pipe) = pipe {
      *mesh.pipe = pipe;
    }
    if !dsets.is_empty() {
      debug_assert!(mesh.dset.len() == dsets.len(), "inconsistent dset length");
      for (dst, src) in mesh.dset.iter_mut().zip(dsets.iter()) {
        if let Some(src) = src {
          *dst = *src
        }
      }
    }
    if !buffers.is_empty() {
      debug_assert!(mesh.buffers.len() == buffers.len(), "inconsistent buffers length");
      for (dst, src) in mesh.buffers.iter_mut().zip(buffers.iter()) {
        if let Some(src) = src {
          *dst = *src
        }
      }
    }
    if let Some(draw) = draw {
      mesh.draw.draw = draw;
    }
  }

  pub fn contains(&self, mesh: MeshId) -> bool {
    self.meshes.contains(mesh.into())
  }

  pub fn remove(&mut self, mesh: MeshId) -> bool {
    if self.meshes.contains(mesh.into()) {
      let m: usize = mesh.into();
      let m = self.meshes[m];
      self.pipes.free(m.pipe, 1);
      self.dsets.free(m.dset.0, m.dset.1);
      self.buffers.free(m.buffers.0, m.buffers.1);
      self.offsets.free(m.buffers.0, m.buffers.1);

      self.meshes.free(mesh.into(), 1);
      true
    } else {
      false
    }
  }

  pub fn clear(&mut self) {
    self.pipes.clear();
    self.dsets.clear();
    self.buffers.clear();
    self.offsets.clear();
    self.meshes.clear();
  }

  pub fn toggle(&mut self, mesh: MeshId, toggle: bool) {
    let m: usize = mesh.into();
    self.meshes[m].toggle = toggle;
  }

  pub fn get<'a>(&'a self, mesh: MeshId) -> DrawMeshRef<'a> {
    let m: usize = mesh.into();
    let m = &self.meshes[m];
    DrawMeshRef {
      pipe: &self.pipes[m.pipe],
      dset: &self.dsets[m.dset.0..m.dset.0 + m.dset.1],
      buffers: &self.buffers[m.buffers.0..m.buffers.0 + m.buffers.1],
      offsets: &self.buffers[m.buffers.0..m.buffers.0 + m.buffers.1],
      draw: &m.draw,
    }
  }

  pub fn iter<'a, T: Iterator<Item = MeshId>>(&'a self, mesh_iter: T) -> DrawPassIterator<'a, T> {
    DrawPassIterator { pass: self, mesh_iter }
  }
}

impl StreamPush for DrawPass {
  fn enqueue(&self, mut cs: CmdBuffer) -> CmdBuffer {
    for d in self.meshes.iter().filter(|d| d.toggle) {
      cs = cs.push(&self.pipes[d.pipe]);
      for ds in self.dsets[d.dset.0..d.dset.0 + d.dset.1].iter() {
        cs = cs.push(ds);
      }
      cs = cs.push(&d.draw);
    }
    cs
  }
}

pub struct DrawPassIterator<'a, T: Iterator<Item = MeshId>> {
  pass: &'a DrawPass,
  mesh_iter: T,
}

impl<'a, T: Iterator<Item = MeshId>> StreamPushMut for DrawPassIterator<'a, T> {
  fn enqueue_mut(&mut self, mut cs: CmdBuffer) -> CmdBuffer {
    for mesh in self.mesh_iter.by_ref() {
      let draw = self.pass.get(mesh);
      cs = cs.push(&draw)
    }
    cs
  }
}
