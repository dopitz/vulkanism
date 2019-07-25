use crate::pipeid::*;
use crate::select::SelectPass;
use crate::select::rects::Rects as SelectRects;
use crate::window::ColumnLayout;
use crate::window::Layout;
use crate::window::RootWindow;
use crate::window::Window;
use font::*;
use std::sync::Arc;
use std::sync::Mutex;
use vk::builder::Buildable;
use vk::cmd::stream::*;
use vk::cmd::CmdPool;
use vk::mem::Handle;
use vk::pass::DrawPass;
use vk::pipes::CachedPipeline;

struct Pass {
  rp: vk::pass::Renderpass,
  fb: Mutex<vk::pass::Framebuffer>,
  draw: Mutex<DrawPass>,
}

struct ImGuiImpl {
  device: vk::Device,
  mem: vk::mem::Mem,

  draw: Pass,
  ub_viewport: Mutex<vk::Buffer>,

  font: Arc<Font>,

  pipes: PipeCache,

  root: Mutex<Option<RootWindow>>,

  rects: SelectRects,
}

impl Drop for ImGuiImpl {
  fn drop(&mut self) {
    self.mem.trash.push_buffer(*self.ub_viewport.lock().unwrap());
  }
}

#[derive(Clone)]
pub struct ImGui {
  select: SelectPass,
  gui: Arc<ImGuiImpl>,
}

impl ImGui {
  pub fn new(
    device: vk::Device,
    copy_queue: vk::Queue,
    cmds: CmdPool,
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
      let mut map = mem.alloc.get_mapped(Handle::Buffer(ub_viewport)).unwrap();
      let data = map.as_slice_mut::<u32>();
      data[0] = extent.width as u32;
      data[1] = extent.height as u32;
    }

    let select = SelectPass::new(device, extent, 1.0, mem.clone());

    let draw = {
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

      let fb = Mutex::new(
        vk::pass::Framebuffer::build_from_pass(&rp, &mut mem.alloc)
          .extent(extent)
          .target(0, target)
          .create(),
      );

      Pass {
        rp,
        fb,
        draw: Mutex::new(DrawPass::new()),
      }
    };

    let pipes = PipeCache::new(&PipeCreateInfo {
      device,
      pass: draw.rp.pass,
      subpass: 0,
      ub_viewport,

      select_pass: select.get_pass(),
      select_subpass: 0,
    });

    let rects = SelectRects::new(device, select.clone(), &pipes[PipeId::SelectRects], mem.clone());

    Self {
      select,
      gui: Arc::new(ImGuiImpl {
        device,
        mem,

        draw,
        ub_viewport: Mutex::new(ub_viewport),

        font,

        pipes,

        root: Mutex::new(None),

        rects,
      }),
    }
  }

  pub fn get_device(&self) -> vk::Device {
    self.gui.device
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

  pub fn get_selectpass(&self) -> SelectPass {
    self.select.clone()
  }
  pub fn get_drawpass<'a>(&'a self) -> std::sync::MutexGuard<'a, DrawPass> {
    self.gui.draw.draw.lock().unwrap()
  }

  pub fn resize(&mut self, size: vk::Extent2D, target: vk::Image) {
    let mut mem = self.gui.mem.clone();
    *self.gui.draw.fb.lock().unwrap() = vk::pass::Framebuffer::build_from_pass(&self.gui.draw.rp, &mut mem.alloc)
      .extent(size)
      .target(0, target)
      .create();

    self.select.resize(size);

    let ub = *self.gui.ub_viewport.lock().unwrap();
    let mut map = mem.alloc.get_mapped(Handle::Buffer(ub)).unwrap();
    let data = map.as_slice_mut::<u32>();
    data[0] = size.width as u32;
    data[1] = size.height as u32;
  }

  pub fn begin_draw(&self, cs: CmdBuffer) -> CmdBuffer {
    let fb = self.gui.draw.fb.lock().unwrap();
    cs.push(&vk::cmd::commands::ImageBarrier::to_color_attachment(fb.images[0]))
      .push(&fb.begin())
      .push(&vk::cmd::commands::Viewport::with_extent(fb.extent))
      .push(&vk::cmd::commands::Scissor::with_extent(fb.extent))
  }
  pub fn end_draw(&self, cs: CmdBuffer) -> CmdBuffer {
    let fb = self.gui.draw.fb.lock().unwrap();
    cs.push(&fb.end())
  }

  pub fn begin(&mut self) -> RootWindow {
    println!("{:?}", self.gui.root.lock().unwrap().as_mut().and_then(|rw| rw.get_select_result()));

    match self.gui.root.lock().unwrap().take() {
      Some(rw) => RootWindow::from_cached(self.clone(), rw),
      None => RootWindow::new(self.clone()),
    }
  }
  pub fn end(&mut self, root: RootWindow) {
    self.gui.root.lock().unwrap().replace(root);
  }
  pub fn begin_window(&mut self) -> Window<ColumnLayout> {
    self.begin_layout(ColumnLayout::default())
  }
  pub fn begin_layout<T: Layout>(&mut self, layout: T) -> Window<T> {
    self.begin().begin_layout(layout)
  }
  pub fn get_size(&self) -> vk::Extent2D {
    self.gui.draw.fb.lock().unwrap().extent
  }
}
