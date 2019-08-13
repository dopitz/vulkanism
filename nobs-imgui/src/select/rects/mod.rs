mod pipeline;

pub use pipeline::Pipeline;
pub use pipeline::Vertex;

use crate::select::SelectId;
use crate::select::SelectPass;
use std::collections::BTreeSet;
use vk;
use vk::cmd::commands::DrawManaged;
use vk::cmd::commands::DrawVertices;
use vk::mem::Handle;
use vk::pass::MeshId;
use vkm::Vec2i;
use vkm::Vec2u;

/// Id type to uniquely identify a sprte for object selection
///
/// We define a separate type for this so that we get more type checking from the compiler.
/// Conversion to and from `usize` is still available with `into()` and `from()`, however we have to conscientously do this.
/// This way it is less likely to accidentally use an `usize` as a RectId that is in fact not.
///
/// We can also forbid certain operations on ids (eg. mul, div, bit logic) because that doesn't make sense for ids.
/// However we allow addition with ***vanilla*** usize and getting the difference between two MeshIds.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct RectId {
  id: usize,
}

impl RectId {
  /// We reserve `usize::max_value` as an invalid id that must not be used.
  pub fn invalid() -> Self {
    Self { id: usize::max_value() }
  }
}

impl Into<usize> for RectId {
  fn into(self) -> usize {
    self.id
  }
}
impl From<usize> for RectId {
  fn from(id: usize) -> Self {
    Self { id }
  }
}

impl std::ops::Add<usize> for RectId {
  type Output = Self;
  fn add(self, rhs: usize) -> Self {
    (self.id + rhs).into()
  }
}
impl std::ops::Sub for RectId {
  type Output = isize;
  fn sub(self, rhs: Self) -> isize {
    self.id as isize - rhs.id as isize
  }
}

/// Manages meshes for drawing rects/billboards in a [SelectPass](../struct.SelectPass.html)
///
/// Rects are rectangular regions on the framebuffer given in pixel coordinates.
///
/// Rects are drawn with individual meshes in the SelectPass. The memory is managed together for all Rects.
/// This means we only have a single vertex buffer. When a mesh is created it is setup to draw a single instance of the vertex buffer.
///
/// We do this, because the gui needs to be shure that draw calles are in order when we draw overlapping gui elements.
/// Ordering of the draw calls might not match the order in the vertex buffer.
pub struct Rects {
  pass: SelectPass,
  mem: vk::mem::Mem,

  vb: vk::Buffer,
  vb_capacity: usize,
  vb_data: Vec<Vertex>,
  vb_free: BTreeSet<usize>,
  vb_dirty: bool,
  meshes: Vec<MeshId>,

  pipe: Pipeline,
}

impl Drop for Rects {
  fn drop(&mut self) {
    self.mem.trash.push_buffer(self.vb);
  }
}

impl Rects {
  /// Creates the manager for the specified select pass.
  ///
  /// [Select](../struct.Select.html) already creates and sets up a manager for Rect object selection and can be used for non gui purposes as well.
  ///
  /// # Arguments
  /// * `pass` - the select pass from which object ids and mesh ids are retrieved
  /// * `pipes` - pipeline manager of [ImGui](../../struct.ImGui.html)
  /// * `mem` - memory manager to allocate vertex buffers
  pub fn new(pass: SelectPass, mem: vk::mem::Mem, ds_viewport: vk::DescriptorSet) -> Self {
    let vb = vk::NULL_HANDLE;

    let pipe = Pipeline::new(pass.get_device(), pass.get_renderpass(), 0, ds_viewport);

    Rects {
      pass,
      mem,

      vb,
      vb_capacity: 0,
      vb_data: Default::default(),
      vb_free: Default::default(),
      vb_dirty: false,
      meshes: Default::default(),
      pipe,
    }
  }

  /// Create a new Rect
  ///
  /// Creating a rect will allocate a new [SelectId](../struct.SelectId.html) and mesh id from the [SelectPass](../struct.SelectPass.html)
  /// The rect will be identified with this id for all futher modifications,
  /// eg. [updating](struct.Rects.html#method.update_rect),
  /// [removing](struct.Rects.html#method.remove),
  /// [accessing](struct.Rects.html#method.get),
  /// or [retrieving the MeshId](struct.Rects.html#method.get_mesh)
  ///
  /// # Arguments
  /// * `pos` - top left position of the rect in pixel coordinates
  /// * `size` - size of the rect in pixels
  ///
  /// # Returs
  /// The object id of the rect. Since this id was allocated from the [SelectPass](../struct.SelectPass.html) we can be sure that no other selectable object with the same id exists.
  pub fn new_rect(&mut self, pos: Vec2i, size: Vec2u) -> RectId {
    let rect = match self.vb_free.iter().next().cloned() {
      Some(i) => {
        self.vb_free.remove(&i);
        i
      }
      None => {
        self.meshes.push(MeshId::invalid());
        self.vb_data.push(Default::default());
        self.vb_data.len() - 1
      }
    };
    self.vb_data[rect].id = self.pass.new_id().into();
    self.meshes[rect] = self.pass.new_mesh(
      self.pipe.bind_pipe,
      &[self.pipe.bind_ds_viewport],
      DrawManaged::new(
        [(self.vb, 0)].iter().into(),
        DrawVertices::with_vertices(4).instance_count(1).first_instance(rect as u32).into(),
      ),
    );
    let rect = RectId::from(rect);
    self.update_rect(rect, pos, size);
    rect
  }

  /// Updates position and size of the rect
  ///
  /// To make changes visible we have to update the vertex buffer with [update](struct.Rects.html#method.update)
  ///
  /// # Arguments
  /// * `i` - id of the rect (returned from [new_rect](struct.Rects.html#method.new_rect))
  /// * `pos` - top left position of the rect in pixel coordinates
  /// * `size` - size of the rect in pixels
  pub fn update_rect(&mut self, i: RectId, pos: Vec2i, size: Vec2u) {
    let i: usize = i.into();
    self.vb_data[i].pos = pos.into();
    self.vb_data[i].size = size.into();
    self.vb_dirty = true;
  }

  /// Removes the rect
  ///
  /// Removes the associated mesh in [SelectPass](../struct.SelectPass.html).
  /// Frees the object id in [SelectPass](../struct.SelectPass.html).
  ///
  /// # Arguments
  /// * `i` - id of the rect (returned from [new_rect](struct.Rects.html#method.new_rect))
  pub fn remove(&mut self, i: RectId) {
    let i: usize = i.into();
    self.pass.remove_mesh(self.meshes[i]);
    self.pass.remove_id(self.vb_data[i].id.into());
    self.vb_free.insert(i);
  }

  /// Gets the Rect that is stored in the vertex buffer
  ///
  /// # Arguments
  /// * `i` - id of the rect (returned from [new_rect](struct.Rects.html#method.new_rect))
  ///
  /// # Returns
  /// Reference to the vertex buffer content
  pub fn get(&self, i: RectId) -> &Vertex {
    let i: usize = i.into();
    &self.vb_data[i]
  }

  /// Gets the associated [SelectId](../struct.SelectId.html)
  ///
  /// # Arguments
  /// * `i` - id of the rect (returned from [new_rect](struct.Rects.html#method.new_rect))
  ///
  /// # Returns
  /// The SelectId
  pub fn get_select_id(&self, i: RectId) -> SelectId {
    let i: usize = i.into();
    SelectId::from(self.vb_data[i].id)
  }

  /// Gets the associated mesh id
  ///
  /// # Arguments
  /// * `i` - id of the rect (returned from [new_rect](struct.Rects.html#method.new_rect))
  ///
  /// # Returns
  /// The MeshId
  pub fn get_mesh(&self, i: RectId) -> MeshId {
    let i: usize = i.into();
    self.meshes[i]
  }

  /// Updates the vertex buffer
  ///
  /// [Updating rects](struct.Rects.html#method.update_rect) will not copy changes to the vertex buffer.
  /// Instead updates are copied in batch with this function.
  ///
  /// We can conservatively call this function even if no changes to the rects have been made, the manager internally tracks if the vertex buffer is dirty and needs updating.
  pub fn update(&mut self) {
    if self.vb_dirty {
      // create new buffer if capacity of cached one is not enough
      if self.vb_data.len() > self.vb_capacity {
        self.mem.trash.push_buffer(self.vb);
        self.vb = vk::NULL_HANDLE;

        vk::mem::Buffer::new(&mut self.vb)
          .vertex_buffer((self.vb_data.len() * std::mem::size_of::<Vertex>()) as vk::DeviceSize)
          .devicelocal(false)
          .bind(&mut self.mem.alloc, vk::mem::BindType::Block)
          .unwrap();

        self.vb_capacity = self.vb_data.len();

        // update the vertex buffer in the draw meshes of the select pass
        for m in self.meshes.iter() {
          self.pass.update_mesh(*m, None, &[], &[Some(self.vb)], None);
        }
      }

      // only copy if not empty
      if !self.vb_data.is_empty() {
        self
          .mem
          .alloc
          .get_mapped(Handle::Buffer(self.vb))
          .unwrap()
          .host_to_device_slice(&self.vb_data);
      }
      self.vb_dirty = false;
    }
  }
}
