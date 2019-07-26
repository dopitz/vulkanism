use crate::rect::Rect;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use vk::builder::Buildable;
use vk::cmd::commands::BindDset;
use vk::cmd::commands::BindPipeline;
use vk::cmd::commands::DrawKind;
use vk::cmd::commands::DrawManaged;
use vk::cmd::commands::RenderpassBegin;
use vk::cmd::commands::RenderpassEnd;
use vk::cmd::commands::Scissor;
use vk::cmd::stream::*;
use vk::mem::Handle;
use vk::pass::DrawMeshRef;
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

  stage: vk::mem::Staging,
  current_pos: vkm::Vec2u,
  dpi: f64,

  free_ids: BTreeSet<Ids>,
}

impl Drop for SelectPassImpl {
  fn drop(&mut self) {
    self.mem.trash.push(Handle::Image(self.fb.images[0]));
  }
}

#[derive(Clone)]
pub struct SelectPass {
  pass: Arc<Mutex<SelectPassImpl>>,
}

impl SelectPass {
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
    fb.set_clear(&[
      vk::ClearValue::build().coloru32([c, c, c, c]).into(),
    ]);

    let stage = vk::mem::Staging::new(mem.clone(), std::mem::size_of::<u32>() as vk::DeviceSize).unwrap();

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

        stage,
        current_pos: vec2!(0),
        dpi,

        free_ids,
      })),
    }
  }

  pub fn resize(&mut self, size: vk::Extent2D) {
    let mut pass = self.pass.lock().unwrap();
    let mut mem = pass.mem.clone();
    mem.alloc.destroy(Handle::Image(pass.fb.images[0]));
    pass.fb = vk::pass::Framebuffer::build_from_pass(&pass.rp, &mut mem.alloc)
      .extent(size)
      .create();
    let c = u32::max_value();
    pass.fb.set_clear(&[
      vk::ClearValue::build().coloru32([c, c, c, c]).into(),
    ]);
  }

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

  pub fn new_mesh(&mut self, pipe: BindPipeline, dsets: &[BindDset], draw: DrawManaged) -> usize {
    self.pass.lock().unwrap().pass.new_mesh(pipe, dsets, draw)
  }

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

  pub fn contains(&self, mesh: usize) -> bool {
    self.pass.lock().unwrap().pass.contains(mesh)
  }

  pub fn remove_mesh(&mut self, mesh: usize) -> bool {
    self.pass.lock().unwrap().pass.remove(mesh)
  }

  pub fn push_query<'a>(&self, q: &'a mut Query) -> PushQuery<'a> {
    q.reset();
    PushQuery {
      pass: self.clone(),
      query: q,
    }
  }

  pub fn get_pass(&self) -> vk::RenderPass {
    self.pass.lock().unwrap().rp.pass
  }
}

pub struct Query {
  meshes: Vec<(usize, Option<Scissor>)>,
  result: Option<u32>,
  dirty: bool,
  stage: vk::mem::Staging,
}

impl Query {
  pub fn new(mem: vk::mem::Mem) -> Self {
    Query {
      meshes: Default::default(),
      result: None,
      dirty: true,
      stage: vk::mem::Staging::new(mem, std::mem::size_of::<u32>() as vk::DeviceSize).unwrap(),
    }
  }

  pub fn clear(&mut self) {
    self.meshes.clear();
    self.result = None;
    self.dirty = false;
  }

  pub fn reset(&mut self) {
    self.result = None;
    self.dirty = true;
  }

  pub fn push(&mut self, mesh: usize, scissor: Option<Scissor>) {
    self.meshes.push((mesh, scissor));
  }

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
          .image_offset(
            vk::Offset3D::build()
              .set(pass.current_pos.x as i32, pass.current_pos.y as i32, 0)
              .into(),
          )
          .image_extent(vk::Extent3D::build().set(1, 1, 1).into()),
      ),
    )
  }
}
