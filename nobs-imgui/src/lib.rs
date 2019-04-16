extern crate cgmath as cgm;
extern crate nobs_vulkanism_headless as vk;

mod font;

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

  fonts: Arc<Mutex<HashMap<FontID, Font>>>,
  pipe_text: Arc<Mutex<Cached<text::Pipeline>>>,

  pub ub: vk::Buffer,

  cs: Option<vk::cmd::Stream>,


  windows: Vec<window::Window>,
}

impl Drop for ImGui {
  fn drop(&mut self) {
    let fonts = self.fonts.lock().unwrap();
    for (_, f) in fonts.iter() {
      vk::DestroyImageView(self.device, f.texview, std::ptr::null());
      vk::DestroySampler(self.device, f.sampler, std::ptr::null());
    }

    for w in self.windows.iter() {
      self.alloc.destroy(w.ub_viewport);
    }
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
  ) -> Self {
    let mut ub = vk::NULL_HANDLE;
    vk::mem::Buffer::new(&mut ub)
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

      fonts: Default::default(),
      pipe_text: Default::default(),

      ub,
      cs: None,

      windows: Default::default(),
    }
  }

  pub fn get_font(&self, font: &FontID) -> Font {
    let mut fonts = self.fonts.lock().unwrap();
    *fonts.entry(font.clone()).or_insert_with(|| Font::new(&font, self))
  }

  pub fn get_pipe_text(&self) -> std::sync::MutexGuard<Cached<text::Pipeline>> {
    let mut p = self.pipe_text.lock().unwrap();
    if !p.is_init() {
      *p = Cached::Init(text::Pipeline::new(self.device, self.pass, self.subpass));
    }
    p
  }

  pub fn resize(&self, extent: vk::Extent2D) {
    let mut map = self.alloc.get_mapped(self.ub).unwrap();
    let data = map.as_slice_mut::<u32>();
    data[0] = extent.width as u32;
    data[1] = extent.height as u32;
  }

  pub fn begin(&mut self, cs: vk::cmd::Stream) -> &mut Self {
    if self.cs.is_none() {
      self.cs = Some(cs);
    }
    self
  }
  pub fn end(&mut self) -> Option<vk::cmd::Stream> {
    self.cs.take()
  }

  pub fn push<T: GuiPush>(&mut self, p: &mut T) -> &mut Self {
    if let Some(cs) = self.cs.take() {
      self.cs = Some(p.enqueue(cs, self))
    }
    self
  }
}



pub trait GuiPush {
  fn enqueue(&mut self, cs: vk::cmd::Stream, gui: &ImGui) -> vk::cmd::Stream;
}

