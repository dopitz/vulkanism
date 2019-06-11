use std::sync::Arc;
use std::sync::Mutex;

use crate::pipeid::*;
use crate::window;
use font::*;
use vk::builder::Buildable;
use vk::cmd::commands::BindDset;
use vk::cmd::commands::BindPipeline;
use vk::cmd::commands::DrawManaged;
use vk::cmd::commands::StreamPush;
use vk::cmd::Stream;
use vk::pass::DrawPass;
use vk::pipes::CachedPipeline;

struct Viewport {
  ub: vk::Buffer,
}

struct ImGuiImpl {
  device: vk::Device,
  copy_queue: vk::Queue,
  cmds: vk::cmd::Pool,
  rp: vk::pass::Renderpass,
  fb: Mutex<vk::pass::Framebuffer>,
  mem: vk::mem::Mem,
  font: Arc<Font>,

  draw: Mutex<DrawPass>,

  pipes: PipeCache,
  ub_viewport: Mutex<vk::Buffer>,
}

impl Drop for ImGuiImpl {
  fn drop(&mut self) {
    self.mem.alloc.destroy(*self.ub_viewport.lock().unwrap());
  }
}

#[derive(Clone)]
pub struct ImGui {
  gui: Arc<ImGuiImpl>,
}

impl ImGui {
  pub fn new(
    device: vk::Device,
    copy_queue: vk::Queue,
    cmds: vk::cmd::Pool,
    extent: vk::Extent2D,
    target: vk::Image,
    mut mem: vk::mem::Mem,
  ) -> Self {
    let font = Arc::new(font::dejavu_mono::new(device, mem.clone(), copy_queue, &cmds));

    let mut ub_viewport = vk::NULL_HANDLE;
    vk::mem::Buffer::new(&mut ub_viewport)
      .uniform_buffer(2 * std::mem::size_of::<f32>() as vk::DeviceSize)
      .devicelocal(false)
      .bind(&mut mem.alloc, vk::mem::BindType::Block)
      .unwrap();

    {
      let mut map = mem.alloc.get_mapped(ub_viewport).unwrap();
      let data = map.as_slice_mut::<u32>();
      data[0] = extent.width as u32;
      data[1] = extent.height as u32;
    }

    let rp = vk::pass::Renderpass::build(device)
      .attachment(
        0,
        vk::AttachmentDescription::build()
          .format(vk::FORMAT_B8G8R8A8_UNORM)
          .initial_layout(vk::IMAGE_LAYOUT_COLOR_ATTACHMENT_OPTIMAL)
          .load_op(vk::ATTACHMENT_LOAD_OP_LOAD),
      )
      .subpass(
        0,
        vk::SubpassDescription::build().bindpoint(vk::PIPELINE_BIND_POINT_GRAPHICS).color(0),
      )
      .dependency(vk::SubpassDependency::build().external(0))
      .create()
      .unwrap();
    let pass = rp.pass;

    let fb = Mutex::new(
      vk::pass::Framebuffer::build_from_pass(&rp, &mut mem.alloc)
        .extent(extent)
        .target(0, target)
        .create(),
    );

    Self {
      gui: Arc::new(ImGuiImpl {
        device,
        copy_queue,
        cmds,
        rp,
        fb,
        mem,
        font,

        draw: Mutex::new(DrawPass::new()),

        pipes: PipeCache::new(&PipeCreateInfo {
          device,
          pass,
          subpass: 0,
          ub_viewport,
        }),
        ub_viewport: Mutex::new(ub_viewport),
      }),
    }
  }

  pub fn get_device(&self) -> vk::Device {
    self.gui.device
  }
  pub fn get_copy_queue(&self) -> vk::Queue {
    self.gui.copy_queue
  }
  pub fn get_cmds(&self) -> vk::cmd::Pool {
    self.gui.cmds.clone()
  }
  pub fn get_mem(&self) -> vk::mem::Mem {
    self.gui.mem.clone()
  }
  pub fn get_font(&self) -> Arc<Font> {
    self.gui.font.clone()
  }
  pub fn get_pipe(&self, id: PipeId) -> &CachedPipeline {
    &self.gui.pipes[id]
  }
  pub fn get_ub_viewport(&self) -> vk::Buffer {
    *self.gui.ub_viewport.lock().unwrap()
  }

  pub fn new_mesh(&self, pipe: BindPipeline, dsets: &[BindDset], draw: DrawManaged) -> usize {
    self.gui.draw.lock().unwrap().new_mesh(pipe, dsets, draw)
  }
  pub fn remove_mesh(&self, mesh: usize) -> bool {
    self.gui.draw.lock().unwrap().remove(mesh)
  }

  pub fn get_meshes<'a>(&'a self) -> std::sync::MutexGuard<'a, DrawPass> {
    self.gui.draw.lock().unwrap()
  }

  pub fn resize(&mut self, size: vk::Extent2D, target: vk::Image) {
    let mut mem = self.gui.mem.clone();
    *self.gui.fb.lock().unwrap() = vk::pass::Framebuffer::build_from_pass(&self.gui.rp, &mut mem.alloc)
      .extent(size)
      .target(0, target)
      .create();

    let ub = *self.gui.ub_viewport.lock().unwrap();
    let mut map = mem.alloc.get_mapped(ub).unwrap();
    let data = map.as_slice_mut::<u32>();
    data[0] = size.width as u32;
    data[1] = size.height as u32;
  }

  pub fn begin_window(&self) -> window::Window {
    let extent = self.gui.fb.lock().unwrap().extent;
    window::Window::new(self.clone()).size(extent.width, extent.height)
  }
  pub fn begin(&self, cs: Stream) -> Stream {
    let fb = self.gui.fb.lock().unwrap();
    cs.push(&vk::cmd::commands::ImageBarrier::to_color_attachment(fb.images[0]))
      .push(&fb.begin())
      .push(&vk::cmd::commands::Viewport::with_extent(fb.extent))
      .push(&vk::cmd::commands::Scissor::with_extent(fb.extent))
  }
  pub fn end(&self, cs: Stream) -> Stream {
    let fb = self.gui.fb.lock().unwrap();
    cs.push(&fb.end())
  }
}
