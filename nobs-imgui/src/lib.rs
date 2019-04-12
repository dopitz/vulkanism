extern crate cgmath as cgm;
extern crate nobs_vulkanism_headless as vk;

mod font;

pub mod text;

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

use font::*;

pub struct CachedPipeline {
  pipe: vk::pipes::Pipeline,
  pool: vk::pipes::DescriptorPool,
}

pub struct PipelineCache {
  cache: Arc<Mutex<HashMap<String, CachedPipeline>>>,
}

impl PipelineCache {
  pub fn add(&mut self, name: &str, p: CachedPipeline) {
    let mut c = self.cache.lock().unwrap();
    match c.get(name) {
      Some(p) => (),
      None => {
        c.insert(name.to_owned(), p);
      }
    }
  }

  //pub fn get(&self, name: &str) -> Option<&mut CachedPipeline> {
  //  let mut c = self.cache.lock().unwrap();
  //  c.get_mut(name)
  //}
}

pub struct ImGui {
  pub device: vk::Device,
  pub queue_copy: vk::Queue,
  pub cmds: vk::cmd::Pool,
  pub pass: vk::RenderPass,
  pub subpass: u32,
  pub alloc: vk::mem::Allocator,

  fonts: Arc<Mutex<HashMap<FontID, Font>>>,
}

impl Drop for ImGui {
  fn drop(&mut self) {
    let fonts = self.fonts.lock().unwrap();
    for (_, f) in fonts.iter() {
      vk::DestroyImageView(self.device, f.texview, std::ptr::null());
      vk::DestroySampler(self.device, f.sampler, std::ptr::null());
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
    ImGui {
      device,
      queue_copy,
      cmds,
      pass,
      subpass,
      alloc,

      fonts: Default::default(),
    }
  }

  pub fn get_font(&self, font: &FontID) -> Font {
    let mut fonts = self.fonts.lock().unwrap();
    *fonts.entry(font.clone()).or_insert_with(|| Font::new(&font, self))
  }

  //pub fn resize() {

  //}

  //fn get_current() -> Window {
  //  screen
  //}
}
