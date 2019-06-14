use crate::cmd::commands::BindDset;
use crate::cmd::commands::BindPipeline;
use crate::cmd::commands::Draw;
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

#[derive(Debug, Clone, Copy)]
pub struct DrawMesh {
  pipe: usize,
  dset: (usize, usize),
  buffers: (usize, usize),
  draw: Draw,
}
pub struct DrawMeshRef<'a> {
  pub pipe: &'a BindPipeline,
  pub dset: &'a [BindDset],
  pub buffers: &'a [vk::Buffer],
  pub offsets: &'a [vk::DeviceSize],
  pub draw: &'a Draw,
}
pub struct DrawMeshRefMut<'a> {
  pub pipe: &'a mut BindPipeline,
  pub dset: &'a mut [BindDset],
  pub buffers: &'a mut [vk::Buffer],
  pub offsets: &'a mut [vk::DeviceSize],
  pub draw: &'a mut Draw,
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
impl<'a> StreamPush for DrawMeshRefMut<'a> {
  fn enqueue(&self, mut cs: CmdBuffer) -> CmdBuffer {
    cs = cs.push(self.pipe);
    for ds in self.dset.iter() {
      cs = cs.push(ds);
    }
    cs.push(self.draw)
  }
}

pub struct DrawPass {
  pipes: BlockAlloc<BindPipeline>,
  dsets: BlockAlloc<BindDset>,
  buffers: BlockAlloc<vk::Buffer>,
  offsets: BlockAlloc<vk::DeviceSize>,

  meshes: BlockAlloc<DrawMesh>,
}

impl DrawPass {
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

  pub fn new_mesh(&mut self, pipe: BindPipeline, dsets: &[BindDset], draw: DrawManaged) -> usize {
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
      pipe,
      dset: (dset, dsets.len()),
      buffers: (buffers, draw.vbs.buffers.len()),
      draw: Draw::new((&self.buffers[beg..end], &self.offsets[beg..end]).into(), draw.draw),
    }]);

    // If buffers have been reallocated we need to make sure the BindVertexBuffers of all meshes are updated
    if realloc_vbs {
      self.update_bindvbs();
    }

    mesh
  }

  pub fn remove(&mut self, mesh: usize) -> bool {
    if self.meshes.contains(mesh) {
      let m = self.meshes[mesh];
      self.pipes.free(m.pipe, 1);
      self.dsets.free(m.dset.0, m.dset.1);
      self.buffers.free(m.buffers.0, m.buffers.1);
      self.offsets.free(m.buffers.0, m.buffers.1);

      self.meshes.free(mesh, 1);
      true
    } else {
      false
    }
  }

  pub fn get<'a>(&'a self, mesh: usize) -> DrawMeshRef<'a> {
    let m = &self.meshes[mesh];
    DrawMeshRef {
      pipe: &self.pipes[m.pipe],
      dset: &self.dsets[m.dset.0..m.dset.0 + m.dset.1],
      buffers: &self.buffers[m.buffers.0..m.buffers.0 + m.buffers.1],
      offsets: &self.buffers[m.buffers.0..m.buffers.0 + m.buffers.1],
      draw: &m.draw,
    }
  }
  pub fn get_mut<'a>(&'a mut self, mesh: usize) -> DrawMeshRefMut<'a> {
    let m = &mut self.meshes[mesh];
    DrawMeshRefMut {
      pipe: &mut self.pipes[m.pipe],
      dset: &mut self.dsets[m.dset.0..m.dset.0 + m.dset.1],
      buffers: &mut self.buffers[m.buffers.0..m.buffers.0 + m.buffers.1],
      offsets: &mut self.offsets[m.buffers.0..m.buffers.0 + m.buffers.1],
      draw: &mut m.draw,
    }
  }
}

impl StreamPush for DrawPass {
  fn enqueue(&self, mut cs: CmdBuffer) -> CmdBuffer {
    for d in self.meshes.iter() {
      cs = cs.push(&self.pipes[d.pipe]);
      for ds in self.dsets[d.dset.0..d.dset.0 + d.dset.1].iter() {
        cs = cs.push(ds);
      }
      cs = cs.push(&d.draw);
    }
    cs
  }
}