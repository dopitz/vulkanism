use crate::rect::Rect;
use std::collections::HashMap;
use vk::builder::Buildable;
use vk::cmd::commands::BindDset;
use vk::cmd::commands::BindPipeline;
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

pub struct SelectPass {
  rp: Renderpass,
  fb: Framebuffer,
  pass: DrawPass,
  mem: vk::mem::Mem,

  stage: vk::mem::Staging,
  current_pos: vkm::Vec2u,
  dpi: f64,
}

impl Drop for SelectPass {
  fn drop(&mut self) {
    self.mem.trash.push(Handle::Image(self.fb.images[0]));
  }
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

    let fb = vk::pass::Framebuffer::build_from_pass(&rp, &mut mem.alloc).extent(extent).create();

    let stage = vk::mem::Staging::new(mem.clone(), std::mem::size_of::<u32>() as vk::DeviceSize).unwrap();

    Self {
      rp,
      fb,
      pass: DrawPass::new(),
      mem,

      stage,
      current_pos: vec2!(0),
      dpi,
    }
  }

  pub fn resize(&mut self, size: vk::Extent2D) {
    self.mem.alloc.destroy(Handle::Image(self.fb.images[0]));
    self.fb = vk::pass::Framebuffer::build_from_pass(&self.rp, &mut self.mem.alloc)
      .extent(size)
      .create();
  }

  pub fn handle_events(&mut self, e: &vk::winit::Event) {
    match e {
      vk::winit::Event::WindowEvent {
        event: vk::winit::WindowEvent::CursorMoved { position, .. },
        ..
      } => self.current_pos = (vec2!(position.x, position.y) * self.dpi).into(),
      vk::winit::Event::WindowEvent {
        event: vk::winit::WindowEvent::HiDpiFactorChanged(dpi),
        ..
      } => self.dpi = *dpi,
      _ => (),
    }

    self.current_pos = vkm::Vec2::clamp(
      self.current_pos,
      vec2!(0),
      vec2!(self.fb.extent.width, self.fb.extent.height) - vec2!(1),
    );
  }

  pub fn new_mesh(&mut self, pipe: BindPipeline, dsets: &[BindDset], draw: DrawManaged) -> usize {
    self.pass.new_mesh(pipe, dsets, draw)
  }

  pub fn contains(&self, mesh: usize) -> bool {
    self.pass.contains(mesh)
  }

  pub fn remove(&mut self, mesh: usize) -> bool {
    self.pass.remove(mesh)
  }

  pub fn query<'a>(&'a self, q: &'a mut Query) -> PushQuery<'a> {
    PushQuery { pass: &self, query: q }
  }

  pub fn get_pass(&self) -> vk::RenderPass {
    self.rp.pass
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
      dirty: false,
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
    self.dirty = false;
  }

  pub fn push(&mut self, mesh: usize, scissor: Option<Scissor>) {
    self.meshes.push((mesh, scissor));
  }

  pub fn get(&mut self) -> Option<u32> {
    if self.dirty {
      let id = self.stage.map().unwrap().as_slice::<u32>()[0];
      self.result = if id == u32::max_value() { None } else { Some(id) };
    }
    self.result
  }
}

pub struct PushQuery<'a> {
  pass: &'a SelectPass,
  query: &'a mut Query,
}

impl<'a> StreamPushMut for PushQuery<'a> {
  fn enqueue_mut(&mut self, cs: CmdBuffer) -> CmdBuffer {
    self.query.reset();

    let fb = &self.pass.fb;
    let mut cs = cs
      .push(&vk::cmd::commands::ImageBarrier::to_color_attachment(fb.images[0]))
      .push(&fb.begin())
      .push(&vk::cmd::commands::Viewport::with_extent(fb.extent))
      .push(&vk::cmd::commands::Scissor::with_extent(fb.extent));

    for q in self.query.meshes.iter() {
      cs = cs.push_if(&q.1).push(&self.pass.pass.get(q.0));
    }

    cs.push(&fb.end()).push(
      &self.query.stage.copy_from_image(
        fb.images[0],
        vk::BufferImageCopy::build()
          .subresource(vk::ImageSubresourceLayers::build().aspect(vk::IMAGE_ASPECT_COLOR_BIT).into())
          .image_offset(
            vk::Offset3D::build()
              .set(self.pass.current_pos.x as i32, self.pass.current_pos.y as i32, 0)
              .into(),
          )
          .image_extent(vk::Extent3D::build().set(1, 1, 1).into()),
      ),
    )
  }
}
