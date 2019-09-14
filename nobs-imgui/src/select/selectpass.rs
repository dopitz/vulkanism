use std::collections::BTreeSet;
use std::sync::Arc;
use std::sync::Mutex;
use vk::builder::Buildable;
use vk::cmd::commands::BindDset;
use vk::cmd::commands::BindPipeline;
use vk::cmd::commands::DrawKind;
use vk::cmd::commands::DrawManaged;
use vk::cmd::commands::Scissor;
use vk::cmd::stream::*;
use vk::mem::Handle;
use vk::pass::DrawPass;
use vk::pass::Framebuffer;
use vk::pass::MeshId;
use vk::pass::Renderpass;

/// Id type to uniquely identify a selectable object
///
/// We define a separate type for this so that we get more type checking from the compiler.
/// Conversion to and from `u32` is still available with `into()` and `from()`, however we have to conscientously do this.
/// This way it is less likely to accidentally use an `u32` as a SelectId that is in fact not.
///
/// We can also forbid certain operations on ids (eg. mul, div, bit logic) because that doesn't make sese for ids.
/// However we allow addition with ***vanilla*** u32 and getting the difference between two SelectIds.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SelectId {
  id: u32,
}

impl SelectId {
  /// We reserve `u32::max_value` as an invalid id that must not be used.
  pub fn invalid() -> Self {
    Self { id: u32::max_value() }
  }
}

impl Into<u32> for SelectId {
  fn into(self) -> u32 {
    self.id
  }
}

impl From<u32> for SelectId {
  fn from(id: u32) -> Self {
    Self { id }
  }
}

impl std::ops::Add<u32> for SelectId {
  type Output = SelectId;
  fn add(self, rhs: u32) -> SelectId {
    (self.id + rhs).into()
  }
}

impl std::ops::Sub for SelectId {
  type Output = i32;
  fn sub(self, rhs: SelectId) -> i32 {
    self.id as i32 - rhs.id as i32
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Ids {
  beg: SelectId,
  end: SelectId,
}
impl PartialOrd for Ids {
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    Some(self.beg.cmp(&other.beg))
  }
}
impl Ord for Ids {
  fn cmp(&self, other: &Self) -> std::cmp::Ordering {
    self.beg.cmp(&other.beg)
  }
}

struct SelectPassImpl {
  rp: Renderpass,
  fb: Framebuffer,
  pass: DrawPass,
  mem: vk::mem::Mem,

  current_pos: vkm::Vec2u,
  dpi: f64,

  free_ids: BTreeSet<Ids>,
}

impl Drop for SelectPassImpl {
  fn drop(&mut self) {
    self.mem.trash.push(Handle::Image(self.fb.images[0]));
  }
}

/// Renderpass for object selection
///
/// This is a wrapper around a DrawPass that renderes [SelectIds](struct.SelectId.html) into an u32 image and lets one retrieve the id over which the mouse pointer is currently at.
/// The pass also takes care of managing ids used to identify objects in the framebuffer.
///
/// Using the pass is thread safe, all hazards are handled internally.
#[derive(Clone)]
pub struct SelectPass {
  pass: Arc<Mutex<SelectPassImpl>>,
}

impl SelectPass {
  /// Creates a new object selection pass.
  ///
  /// # Arguments
  /// * `device` - the vulkan device handle
  /// * `extent` - size of the render target
  /// * `dpi` - winit dpi factor of the window, that is needed since mouse pointer coordinates are given in logical coordinates by the window events.
  /// * `mem` - memory manager to allocate framebuffers
  pub fn new(device: vk::Device, extent: vk::Extent2D, dpi: f64, mem: vk::mem::Mem) -> Self {
    let mut mem = mem.clone();
    let rp = vk::pass::Renderpass::build(device)
      .attachment(
        0,
        vk::AttachmentDescription::build()
          .format(vk::FORMAT_R32_UINT)
          .initial_layout(vk::IMAGE_LAYOUT_COLOR_ATTACHMENT_OPTIMAL),
      )
      .subpass(
        0,
        vk::SubpassDescription::build().bindpoint(vk::PIPELINE_BIND_POINT_GRAPHICS).color(0),
      )
      .dependency(vk::SubpassDependency::build().external(0))
      .create()
      .unwrap();

    let mut fb = vk::pass::Framebuffer::build_from_pass(&rp, &mut mem.alloc).extent(extent).create();
    let c = SelectId::invalid().into();
    fb.set_clear(&[vk::ClearValue::build().coloru32([c, c, c, c]).into()]);

    let mut free_ids: BTreeSet<Ids> = Default::default();
    free_ids.insert(Ids {
      beg: SelectId::from(0),
      end: SelectId::invalid().into(),
    });

    Self {
      pass: Arc::new(Mutex::new(SelectPassImpl {
        rp,
        fb,
        pass: DrawPass::new(),
        mem,

        current_pos: vec2!(0),
        dpi,

        free_ids,
      })),
    }
  }

  /// Resizes the framebuffer of this pass
  pub fn resize(&mut self, size: vk::Extent2D) {
    let mut pass = self.pass.lock().unwrap();
    let mut mem = pass.mem.clone();
    mem.alloc.destroy(Handle::Image(pass.fb.images[0]));
    pass.fb = vk::pass::Framebuffer::build_from_pass(&pass.rp, &mut mem.alloc)
      .extent(size)
      .create();
    let c = u32::max_value();
    pass.fb.set_clear(&[vk::ClearValue::build().coloru32([c, c, c, c]).into()]);
  }

  /// Handle window events
  ///
  /// The selection pass handles Window events for mouse movement to update the internal mouse pointer position.
  pub fn handle_events(&mut self, e: &vk::winit::Event) {
    let mut pass = self.pass.lock().unwrap();

    match e {
      vk::winit::Event::WindowEvent {
        event: vk::winit::WindowEvent::CursorMoved { position, .. },
        ..
      } => pass.current_pos = (vec2!(position.x, position.y) * pass.dpi).into(),
      vk::winit::Event::WindowEvent {
        event: vk::winit::WindowEvent::HiDpiFactorChanged(dpi),
        ..
      } => pass.dpi = *dpi,
      _ => (),
    }

    pass.current_pos = vkm::Vec2::clamp(
      pass.current_pos,
      vec2!(0),
      vec2!(pass.fb.extent.width, pass.fb.extent.height) - vec2!(1),
    );
  }

  /// Create a new id for object selection
  ///
  /// [SelectIds](struct.SelectId.html) are completely unrelated to the mesh id ([new_mesh](struct.SelectPass.html#method.new_mesh)).
  /// A single mesh can choose write multiple SelectIds, eg. if rendered with instancing.
  ///
  /// We can not ensure that rendered meshes actually only use the SelectIds they are supposed to.
  /// However, managing the SelectIds centralized in the SelectPass at least ensures that there are no overlapps when ids are distributed.
  ///
  /// # Returns
  /// The id to identify an object on the framebuffer
  pub fn new_id(&self) -> SelectId {
    self.new_ids(1)
  }
  /// Create several ids for object selection in a block
  ///
  /// # Arguments
  /// * `count` - Number of ids in this block
  ///
  /// # Returns
  /// The first [SelectId](struct.SelectId.html) of the block. Ids in the block are continuous meaning the (inclusive) last id in this block is `result + (count - 1)`.
  pub fn new_ids(&self, count: u32) -> SelectId {
    // find and remove block with enough free ids
    let free = &mut self.pass.lock().unwrap().free_ids;
    let ids = *free.iter().find(|ids| ids.beg + count <= ids.end).unwrap();
    free.remove(&ids);

    // add unused ids back
    if ids.beg + count < ids.end {
      free.insert(Ids {
        beg: ids.beg + count,
        end: ids.end,
      });
    }

    // return first id in the block
    ids.beg
  }
  /// Free [SelectId](struct.SelectId.html), so that it may be reused in a future [new_id](struct.SelectPass.html#method.new_id)
  ///
  /// # Arguments
  /// * `id` - id to be freed
  pub fn remove_id(&self, id: SelectId) {
    self.remove_ids(id, 1);
  }
  /// Frees multiple [SelectIds](struct.SelectId.html) in a block.
  ///
  /// Note that when ids ar allocated using [new_ids](struct.SelectPass.html#method.new_ids) we are note forced to free them with the corresponding `remove_ids`.
  /// We can free single ids from block allocated ids.
  /// We can also free multiple ids in a block that have been allocated in multiple calls of [new_id](struct.SelectPass.html#method.new_id)
  ///
  /// # Arguments
  /// * `id` - first id to be freed
  /// * `count` - number of ids to free
  pub fn remove_ids(&self, id: SelectId, count: u32) {
    let mut f = Ids { beg: id, end: id + count };

    let free = &mut self.pass.lock().unwrap().free_ids;
    loop {
      // find intersecting free block
      match free.iter().find(|i| id + count <= i.beg && id <= i.end).cloned() {
        Some(ids) => {
          free.remove(&ids);
          f.beg = SelectId::min(ids.beg, f.beg);
          f.end = SelectId::max(ids.end, f.end + count);
        }
        None => break,
      }
    }
  }

  /// Create a mesh in the DrawPass
  ///
  /// See [new_mesh](https://docs.rs/nobs-vulkanism-headless/0.1.0/nobs_vulkanism_headless/pass/struct.DrawPass.html#method.new_mesh) for details.
  ///
  /// # Returns
  /// The MeshId
  pub fn new_mesh(&self, pipe: BindPipeline, dsets: &[BindDset], draw: DrawManaged) -> MeshId {
    self.pass.lock().unwrap().pass.new_mesh(pipe, dsets, draw)
  }

  /// Update the mesh in the DrawPass
  ///
  /// See [update_mesh](https://docs.rs/nobs-vulkanism-headless/0.1.0/nobs_vulkanism_headless/pass/struct.DrawPass.html#method.update_mesh) for details.
  pub fn update_mesh(
    &self,
    mesh: MeshId,
    pipe: Option<BindPipeline>,
    dsets: &[Option<BindDset>],
    buffers: &[Option<vk::Buffer>],
    draw: Option<DrawKind>,
  ) {
    self.pass.lock().unwrap().pass.update_mesh(mesh, pipe, dsets, buffers, draw);
  }

  /// Checks if a there is a mesh with the specified id
  ///
  /// See [contains](https://docs.rs/nobs-vulkanism-headless/0.1.0/nobs_vulkanism_headless/pass/struct.DrawPass.html#method.contains) for details.
  pub fn contains(&self, mesh: MeshId) -> bool {
    self.pass.lock().unwrap().pass.contains(mesh)
  }

  /// Removes the mesh with the specified mesh id if present
  ///
  /// See [remove_mesh](https://docs.rs/nobs-vulkanism-headless/0.1.0/nobs_vulkanism_headless/pass/struct.DrawPass.html#method.remove_mesh) for details.
  pub fn remove_mesh(&self, mesh: MeshId) -> bool {
    self.pass.lock().unwrap().pass.remove(mesh)
  }

  /// Run the query on this render pass
  ///
  /// Runs the query, retrieves the object id under the current mouse pointer position.
  /// See [push_query_with_pos](struct.SelectPass.html#method.push_query_with_pos).
  pub fn push_query<'a>(&self, q: &'a mut Query) -> PushQuery<'a> {
    let pos = self.pass.lock().unwrap().current_pos;
    self.push_query_with_pos(q, pos)
  }

  /// Run the query on this render pass
  ///
  /// [Resets](struct.Query.html#method.reset) the query and returns a PushQury.
  ///
  /// The PushQuery is a helper struct to push a [Query](struct.Query.html) into a command buffer
  ///
  /// Enqueuing a Query into a command buffer will
  /// 1. reset the Query
  /// 2. render meshes in order in which they were added
  /// 3. copy the result into the query's staging buffer
  ///
  /// # Arguments
  /// * `q` - mutable refenece to the Query that is to be run. The query will render all its recorded meshes in order and hold the selection result
  /// * `pos` - position in pixel coordinates to read back from the render target
  pub fn push_query_with_pos<'a>(&self, q: &'a mut Query, pos: vkm::Vec2u) -> PushQuery<'a> {
    let extent = self.pass.lock().unwrap().fb.extent;
    let pos = vkm::Vec2::clamp(pos, vec2!(0), vec2!(extent.width, extent.height) - vec2!(1));

    q.reset();
    PushQuery {
      pass: self.clone(),
      query: q,
      pos,
    }
  }

  /// Get the vulkan handle of the render pass
  pub fn get_renderpass(&self) -> vk::RenderPass {
    self.pass.lock().unwrap().rp.pass
  }

  /// Get the vulkan device of this pass
  pub fn get_device(&self) -> vk::Device {
    self.pass.lock().unwrap().rp.device
  }

  /// Converts `pos` into winit logical coordinates
  ///
  /// Winit uses a dpi factor, but we usually want pixel coordinates.
  /// Use this function to convert a logical position from winit into pixel coordinate values.
  pub fn logic_to_real_position(&self, pos: vk::winit::dpi::LogicalPosition) -> vkm::Vec2i {
    (vec2!(pos.x, pos.y) * self.pass.lock().unwrap().dpi).into()
  }

  /// Converts `pos` into winit logical coordinates
  ///
  /// Winit uses a dpi factor, but we usually want pixel coordinates.
  /// Use this function to convert pixel coordinate values with the current dpi settings
  pub fn real_to_logic_position(&self, pos: vkm::Vec2u) -> vk::winit::dpi::LogicalPosition {
    let pos = pos.into() / self.pass.lock().unwrap().dpi;
    vk::winit::dpi::LogicalPosition { x: pos.x, y: pos.y }
  }

  /// Gets the current mouse cursor position
  ///
  /// The cursor position is tracked in pixel coordinates with `(0,0)` in top left corner
  pub fn get_current_position(&self) -> vkm::Vec2u {
    self.pass.lock().unwrap().current_pos
  }
}

/// Query that collects meshes and reads back the SelectId.
///
/// The query collects mesh ids and optionally scissor rects.
/// All mesh ids must stem from the same [SelectPass](struct.SelectPass.html).
pub struct Query {
  meshes: Vec<(MeshId, Option<Scissor>)>,
  result: Option<SelectId>,
  dirty: bool,
  stage: vk::mem::Staging,
}

impl Query {
  /// Creates a new querry.
  ///
  /// Creates a staging buffer to read back the selection result.
  /// The inital selection result is set `None`.
  /// No meshes are added to the query.
  pub fn new(mem: vk::mem::Mem) -> Self {
    Query {
      meshes: Default::default(),
      result: None,
      dirty: true,
      stage: vk::mem::Staging::new(mem, std::mem::size_of::<u32>() as vk::DeviceSize).unwrap(),
    }
  }

  /// Clears meshes.
  ///
  /// Does not change the capacity of the query so no reallocation is needed when meshes are pushed into the query again.
  pub fn clear(&mut self) {
    self.meshes.clear();
  }

  /// Resets the query result to `None`
  ///
  /// Sets the cached selction result back to `None`.
  /// This will also force [get](struct.Query.html#method.get) to read back the result from the staging buffer again.
  pub fn reset(&mut self) {
    self.result = None;
    self.dirty = true;
  }

  /// Adds a mesh and scissor rect to the query
  ///
  /// # Arguments
  /// * `mesh` - the MeshId to render the SelectIds into the framebuffer
  /// * `scissor` - if a scissor command is specified this scissor rect will be used befor drawing `mesh`
  pub fn push(&mut self, mesh: MeshId, scissor: Option<Scissor>) {
    self.meshes.push((mesh, scissor));
  }

  /// Retrievs the query's result
  ///
  /// Reads back the result from the staging buffer the first time it is called.
  /// After that the query result will be cached and no GPU readback is needed.
  ///
  /// Once [reset](struct.Query.html#method.reset) is called the cached result will be invalidated.
  pub fn get(&mut self) -> Option<SelectId> {
    if self.dirty {
      let id = self.stage.map().unwrap().as_slice::<u32>()[0];
      self.result = if id == u32::max_value() { None } else { Some(SelectId::from(id)) };
      self.dirty = false;
    }
    self.result
  }
}

pub struct PushQuery<'a> {
  pass: SelectPass,
  query: &'a mut Query,
  pos: vkm::Vec2u,
}

impl<'a> StreamPushMut for PushQuery<'a> {
  fn enqueue_mut(&mut self, cs: CmdBuffer) -> CmdBuffer {
    self.query.reset();

    let pass = self.pass.pass.lock().unwrap();
    let fb = &pass.fb;
    let mut cs = cs
      .push(&vk::cmd::commands::ImageBarrier::to_color_attachment(fb.images[0]))
      .push(&fb.begin())
      .push(&vk::cmd::commands::Viewport::with_extent(fb.extent))
      .push(&vk::cmd::commands::Scissor::with_extent(fb.extent));

    for q in self.query.meshes.iter() {
      cs = cs.push_if(&q.1).push(&pass.pass.get(q.0));
    }

    cs.push(&fb.end()).push(
      &self.query.stage.copy_from_image(
        fb.images[0],
        vk::BufferImageCopy::build()
          .subresource(vk::ImageSubresourceLayers::build().aspect(vk::IMAGE_ASPECT_COLOR_BIT).into())
          .image_offset(vk::Offset3D::build().set(self.pos.x as i32, self.pos.y as i32, 0).into())
          .image_extent(vk::Extent3D::build().set(1, 1, 1).into()),
      ),
    )
  }
}
