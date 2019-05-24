use font::*;

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

use crate::cachedpipeline::*;
use crate::window;

struct Viewport {
  ub: vk::Buffer,
  cmd: vk::cmd::commands::Viewport,
}

struct ImGuiImpl {
  pub device: vk::Device,
  pub copy_queue: vk::Queue,
  pub cmds: vk::cmd::Pool,
  pub pass: vk::RenderPass,
  pub subpass: u32,
  pub mem: vk::mem::Mem,
  pub font: Arc<Font>,

  pipes: Mutex<HashMap<String, Arc<CachedPipeline>>>,
  vp: Mutex<Viewport>,
}

impl Drop for ImGuiImpl {
  fn drop(&mut self) {
    self.mem.alloc.destroy(self.vp.lock().unwrap().ub);
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
    pass: vk::RenderPass,
    subpass: u32,
    mut mem: vk::mem::Mem,
  ) -> Self {
    let font = Arc::new(font::dejavu_mono::new(device, mem.clone(), copy_queue, &cmds));

    let mut ub_viewport = vk::NULL_HANDLE;
    vk::mem::Buffer::new(&mut ub_viewport)
      .uniform_buffer(2 * std::mem::size_of::<f32>() as vk::DeviceSize)
      .devicelocal(false)
      .bind(&mut mem.alloc, vk::mem::BindType::Block)
      .unwrap();

    Self {
      gui: Arc::new(ImGuiImpl {
        device,
        copy_queue,
        cmds,
        pass,
        subpass,
        mem,
        font,

        pipes: Mutex::new(HashMap::new()),
        vp: Mutex::new(Viewport {
          ub: ub_viewport,
          cmd: vk::cmd::commands::Viewport::with_size(0.0, 0.0),
        }),
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
  pub fn get_pass(&self) -> vk::RenderPass {
    self.gui.pass
  }
  pub fn get_subpass(&self) -> u32 {
    self.gui.subpass
  }
  pub fn get_mem(&self) -> vk::mem::Mem {
    self.gui.mem.clone()
  }
  pub fn get_font(&self) -> Arc<Font> {
    self.gui.font.clone()
  }
  pub fn get_pipeline<T: CacheablePipeline>(&self) -> T {
    self.get_pipeline_setup(|_| {})
  }
  pub fn get_pipeline_setup<T: CacheablePipeline, F: FnOnce(&mut [vk::DescriptorSet])>(&self, setup: F) -> T {
    let gui = &self.gui;
    let mut pipes = gui.pipes.lock().unwrap();
    let ident = T::get_ident();
    let p = match pipes.get_mut(ident) {
      Some(p) => p.clone(),
      None => {
        let mut p = T::create_cache(gui.device, gui.pass, gui.subpass);
        if let Some((_, ref mut dsets)) = p.shared {
          setup(dsets);
        }
        let p = Arc::new(p);
        pipes.insert(ident.to_string(), p.clone());
        p
      }
    };
    T::from_cache(p)
  }
  pub fn get_ub_viewport(&self) -> vk::Buffer {
    self.gui.vp.lock().unwrap().ub
  }
  pub fn get_viewport(&self) -> vk::cmd::commands::Viewport {
    self.gui.vp.lock().unwrap().cmd
  }

  pub fn resize(&mut self, extent: vk::Extent2D) {
    let mut vp = self.gui.vp.lock().unwrap();
    let mut map = self.gui.mem.alloc.get_mapped(vp.ub).unwrap();
    let data = map.as_slice_mut::<u32>();
    data[0] = extent.width as u32;
    data[1] = extent.height as u32;
    vp.cmd = vk::cmd::commands::Viewport::with_extent(extent);
  }

  pub fn begin_window<'a>(&self) -> window::Window<'a> {
    let (ub, vp) = {
      let vp = self.gui.vp.lock().unwrap();
      (vp.ub, vp.cmd.vp)
    };

    window::Window::new(self.gui.device, ub).size(vp.width as u32, vp.height as u32)
  }
}

impl<'a> vk::cmd::commands::StreamPush for ImGui {
  fn enqueue(&self, cs: vk::cmd::Stream) -> vk::cmd::Stream {
    self.gui.mem.trash.clean();
    cs.push(&self.gui.vp.lock().unwrap().cmd)
  }
}
