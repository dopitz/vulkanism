extern crate nobs_vulkanism_headless as vk;
#[macro_use]
extern crate nobs_vkmath as vkm;
extern crate freetype;
extern crate nobs_imgui_font as fnt;

mod font;

pub mod sizebounds;
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
  pub alloc: vk::mem::Allocator,
  pub unused: vk::mem::UnusedResources,

  fonts: Arc<Mutex<HashMap<FontID, Arc<Font>>>>,
  pipe_text: Arc<Mutex<Cached<text::Pipeline>>>,

  pub ub_viewport: vk::Buffer,
  pub viewport: vk::cmd::commands::Viewport,
}

impl Drop for ImGui {
  fn drop(&mut self) {
    let fonts = self.fonts.lock().unwrap();
    for (_, f) in fonts.iter() {
      vk::DestroyImageView(self.device, f.texview, std::ptr::null());
      vk::DestroySampler(self.device, f.sampler, std::ptr::null());
    }

    self.alloc.destroy(self.ub_viewport);
  }
}

mod dejavu {
  use crate::font::Char;
  use crate::font::Font;
  use crate::ImGui;

  fnt::make_font! {
    font = "dejavu/DejaVuSans.ttf",
    margin = 32,
    char_height = 5,
    dump = "src/dejavk.rs",
  }
}

impl ImGui {
  pub fn new(
    device: vk::Device,
    queue_copy: vk::Queue,
    cmds: vk::cmd::Pool,
    pass: vk::RenderPass,
    subpass: u32,
    alloc: vk::mem::Allocator,
    inflight: usize,
  ) -> Self {
    let mut ub_viewport = vk::NULL_HANDLE;
    vk::mem::Buffer::new(&mut ub_viewport)
      .uniform_buffer(2 * std::mem::size_of::<f32>() as vk::DeviceSize)
      .devicelocal(false)
      .bind(&mut alloc.clone(), vk::mem::BindType::Block)
      .unwrap();

    ImGui {
      device,
      queue_copy,
      cmds,
      pass,
      subpass,
      alloc,
      unused: vk::mem::UnusedResources::new(inflight),

      fonts: Default::default(),
      pipe_text: Default::default(),

      ub_viewport,
      viewport: vk::cmd::commands::Viewport::with_size(0.0, 0.0),
    }
  }

  pub fn get_font(&self, font: &FontID) -> Arc<Font> {
    let mut fonts = self.fonts.lock().unwrap();
    fonts
      .entry(font.clone())
      .or_insert_with(|| Arc::new(Font::new(&font, self)))
      .clone()
  }

  pub fn get_pipe_text(&self) -> std::sync::MutexGuard<Cached<text::Pipeline>> {
    let mut p = self.pipe_text.lock().unwrap();
    if !p.is_init() {
      *p = Cached::Init(text::Pipeline::new(self.device, self.pass, self.subpass));
    }
    p
  }

  pub fn resize(&mut self, extent: vk::Extent2D) {
    let mut map = self.alloc.get_mapped(self.ub_viewport).unwrap();
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
    self.unused.free(self.alloc.clone());
    cs.push(&self.viewport)
  }
}
