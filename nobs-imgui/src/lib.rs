extern crate nobs_vulkanism_headless as vk;
#[macro_use]
extern crate nobs_vkmath as vkm;
extern crate freetype;
extern crate nobs_imgui_font as font;

//mod font;

pub mod rect;
pub mod text;
pub mod window;

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

use font::*;

pub enum Cached<T> {
  Init(T),
  Uninit,
}

impl<T> Default for Cached<T> {
  fn default() -> Self {
    Cached::Uninit
  }
}

impl<T> std::ops::DerefMut for Cached<T> {
  fn deref_mut(&mut self) -> &mut T {
    self.get_mut()
  }
}
impl<T> std::ops::Deref for Cached<T> {
  type Target = T;
  fn deref(&self) -> &T {
    self.get()
  }
}

impl<T> Cached<T> {
  pub fn get(&self) -> &T {
    match self {
      Cached::Init(t) => t,
      Cached::Uninit => panic!("AOEUAOUE"),
    }
  }

  pub fn get_mut(&mut self) -> &mut T {
    match self {
      Cached::Init(t) => t,
      Cached::Uninit => panic!("AOEUAOUE"),
    }
  }

  pub fn is_init(&self) -> bool {
    match self {
      Cached::Init(_) => true,
      Cached::Uninit => false,
    }
  }
}

pub struct ImGui {
  pub device: vk::Device,
  pub queue_copy: vk::Queue,
  pub cmds: vk::cmd::Pool,
  pub pass: vk::RenderPass,
  pub subpass: u32,
  pub mem: vk::mem::Mem,
  pub font: Arc<Font>,

  pipes: Arc<Mutex<HashMap<String, Arc<(vk::pipes::Pipeline, vk::pipes::descriptor::Pool)>>>>,
  pipe_text: Arc<Mutex<Cached<text::Pipeline>>>,

  pub ub_viewport: vk::Buffer,
  pub viewport: vk::cmd::commands::Viewport,
}

impl Drop for ImGui {
  fn drop(&mut self) {
    self.mem.alloc.destroy(self.ub_viewport);
  }
}

impl ImGui {
  pub fn new(
    device: vk::Device,
    queue_copy: vk::Queue,
    cmds: vk::cmd::Pool,
    pass: vk::RenderPass,
    subpass: u32,
    mem: vk::mem::Mem,
  ) -> Self {
    let mut mem = mem.clone();
    let font = Arc::new(font::dejavu_mono::new(device, &mem.alloc, queue_copy, &cmds));

    let mut ub_viewport = vk::NULL_HANDLE;
    vk::mem::Buffer::new(&mut ub_viewport)
      .uniform_buffer(2 * std::mem::size_of::<f32>() as vk::DeviceSize)
      .devicelocal(false)
      .bind(&mut mem.alloc, vk::mem::BindType::Block)
      .unwrap();

    ImGui {
      device,
      queue_copy,
      cmds,
      pass,
      subpass,
      mem,
      font,

      pipes: Arc::new(Mutex::new(HashMap::new())),
      pipe_text: Default::default(),

      ub_viewport,
      viewport: vk::cmd::commands::Viewport::with_size(0.0, 0.0),
    }
  }

  pub fn get_font(&self) -> Arc<Font> {
    self.font.clone()
  }

  pub fn get_pipeline<F: FnOnce(&ImGui) -> (vk::pipes::Pipeline, vk::pipes::descriptor::Pool)>(
    &self,
    ident: &str,
    create: F,
  ) -> Arc<(vk::pipes::Pipeline, vk::pipes::descriptor::Pool)> {
    let mut p = self.pipes.lock().unwrap();
    match p.get_mut(ident) {
      Some(x) => x.clone(),
      None => {
        let x = Arc::new(create(&self));
        p.insert(ident.to_string(), x.clone());
        x
      }
    }
  }

  pub fn get_pipe_text(&self) -> std::sync::MutexGuard<Cached<text::Pipeline>> {
    let mut p = self.pipe_text.lock().unwrap();
    if !p.is_init() {
      *p = Cached::Init(text::Pipeline::new(self.device, self.pass, self.subpass));
    }
    p
  }

  pub fn resize(&mut self, extent: vk::Extent2D) {
    let mut map = self.mem.alloc.get_mapped(self.ub_viewport).unwrap();
    let data = map.as_slice_mut::<u32>();
    data[0] = extent.width as u32;
    data[1] = extent.height as u32;
    self.viewport = vk::cmd::commands::Viewport::with_extent(extent);
  }

  pub fn begin_window<'a>(&self) -> window::Window<'a> {
    window::Window::new(self.device, self.ub_viewport).size(self.viewport.vp.width as u32, self.viewport.vp.height as u32)
  }
}

impl<'a> vk::cmd::commands::StreamPush for ImGui {
  fn enqueue(&self, cs: vk::cmd::Stream) -> vk::cmd::Stream {
    self.mem.trash.clean();
    cs.push(&self.viewport)
  }
}
