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
use vk::pass::Renderpass;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Ids {
  beg: u32,
  end: u32,
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
/// This is a wrapper around a draw pass that renderes object ids to a u32 image and lets one retrieve the id over which the mouse pointer is currently at.
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
    let c = u32::max_value();
    fb.set_clear(&[vk::ClearValue::build().coloru32([c, c, c, c]).into()]);

    let mut free_ids: BTreeSet<Ids> = Default::default();
    free_ids.insert(Ids {
      beg: 0,
      end: u32::max_value(),
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
  /// Object ids are completely unrelated to the mesh id ([new_mesh](struct.SelectPass.html#method.new_mesh)).
  /// A single mesh can write multiple object ids, eg. if rendered with instancing.
  ///
  /// We can not ensure that rendered meshes actually only use the object ids they are supposed to.
  ///
  /// Managing the object ids centralized in the SelectPass at least ensures that there are no overlapps when ids are distributed.
  ///
  /// # Returns
  /// The id to identify an object on the framebuffer
  pub fn new_id(&mut self) -> u32 {
    let free = &mut self.pass.lock().unwrap().free_ids;
    let ids = *free.iter().next().unwrap();
    free.remove(&ids);
    if ids.end - ids.beg > 1 {
      free.insert(Ids {
        beg: ids.beg + 1,
        end: ids.end,
      });
    }
    ids.beg
  }
  /// Create several ids for object selection on a block
  ///
  /// # Arguments
  /// * `count` - Number of ids in this block
  ///
  /// # Returns
  /// The first id of the block. Ids in the block are continuous meaning the last id in this block is `result + count - 1`.
  pub fn new_ids(&mut self, count: u32) -> u32 {
    let free = &mut self.pass.lock().unwrap().free_ids;
    let ids = *free.iter().find(|ids| ids.end - ids.beg >= count).unwrap();
    free.remove(&ids);
    if ids.end - ids.beg > count {
      free.insert(Ids {
        beg: ids.beg + count,
        end: ids.end,
      });
    }
    ids.beg
  }
  /// Free id, so that it may be reused in a [new_id](struct.SelectPass.html#method.new_id)
  ///
  /// # Arguments
  /// * `id` - id to be freed
  pub fn remove_id(&mut self, id: u32) {
    let free = &mut self.pass.lock().unwrap().free_ids;
    let mut ids = *free.iter().find(|i| id == i.beg - 1 || id == i.end).unwrap();
    free.remove(&ids);
    if ids.beg - 1 == id {
      ids.beg -= id;
    }
    if ids.end == id {
      ids.end = id + 1;
    }
    free.insert(ids);
  }
  pub fn remove_ids(&mut self, id: u32, count: u32) {
    let free = &mut self.pass.lock().unwrap().free_ids;
    let mut ids = *free.iter().find(|i| i.beg >= id + count && id <= i.end).unwrap();
    free.remove(&ids);
    ids.beg = u32::min(ids.beg, id);
    ids.end = u32::max(ids.end, id + count);
    free.insert(ids);
  }

  /// Create a mesh in the DrawPass
  ///
  /// See [new_mesh](https://docs.rs/nobs-vulkanism-headless/0.1.0/nobs_vulkanism_headless/pass/struct.DrawPass.html#method.new_mesh) for details.
  ///
  /// # Returns
  /// The mesh id
  pub fn new_mesh(&mut self, pipe: BindPipeline, dsets: &[BindDset], draw: DrawManaged) -> usize {
    self.pass.lock().unwrap().pass.new_mesh(pipe, dsets, draw)
  }

  /// Update the mesh in the DrawPass
  ///
  /// See [update_mesh](https://docs.rs/nobs-vulkanism-headless/0.1.0/nobs_vulkanism_headless/pass/struct.DrawPass.html#method.update_mesh) for details.
  pub fn update_mesh(
    &mut self,
    mesh: usize,
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
  pub fn contains(&self, mesh: usize) -> bool {
    self.pass.lock().unwrap().pass.contains(mesh)
  }

  /// Removes the mesh with the specified mesh id if present
  ///
  /// See [remove_mesh](https://docs.rs/nobs-vulkanism-headless/0.1.0/nobs_vulkanism_headless/pass/struct.DrawPass.html#method.remove_mesh) for details.
  pub fn remove_mesh(&mut self, mesh: usize) -> bool {
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
}

/// Query that collects meshes and reads back the selected id.
///
/// The query collects mesh ids and optionally scissor rects.
/// All mesh ids must stem from the same [SelectPass](struct.SelectPass.html).
pub struct Query {
  meshes: Vec<(usize, Option<Scissor>)>,
  result: Option<u32>,
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
  /// * `mesh` - the mesh id
  /// * `scissor` - if a scissor command is specified this scissor rect will be used befor drawing `mesh`
  pub fn push(&mut self, mesh: usize, scissor: Option<Scissor>) {
    self.meshes.push((mesh, scissor));
  }

  /// Retrievs the query's result
  ///
  /// Reads back the result from the staging buffer the first time it is called.
  /// After that the query result will be cached and no GPU readback is needed.
  ///
  /// Once [reset](struct.Query.html#method.reset) is called the cached result will be invalidated.
  pub fn get(&mut self) -> Option<u32> {
    if self.dirty {
      let id = self.stage.map().unwrap().as_slice::<u32>()[0];
      self.result = if id == u32::max_value() { None } else { Some(id) };
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
