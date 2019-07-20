use std::collections::HashMap;
use vk;
use vkm;

#[derive(Clone, Copy, Debug)]
pub struct Char {
  pub tex00: vkm::Vec2f,
  pub tex11: vkm::Vec2f,
  pub size: vkm::Vec2f,
  pub bearing: vkm::Vec2f,
  pub advance: vkm::Vec2f,
}

impl std::ops::Mul<f32> for Char {
  type Output = Char;
  fn mul(self, s: f32) -> Self {
    Char {
      size: self.size * s,
      bearing: self.bearing * s,
      advance: self.advance * s,
      ..self
    }
  }
}

pub struct Font {
  device: vk::Device,
  mem: vk::mem::Mem,

  pub tex: vk::Image,
  pub texview: vk::ImageView,
  pub sampler: vk::Sampler,

  pub chars: HashMap<char, Char>,
}

impl Drop for Font {
  fn drop(&mut self) {
    self.mem.trash.push_image(self.tex);
    vk::DestroyImageView(self.device, self.texview, std::ptr::null());
    vk::DestroySampler(self.device, self.sampler, std::ptr::null());
  }
}

impl Font {
  pub fn new(
    device: vk::Device,
    mem: vk::mem::Mem,
    tex: vk::Image,
    texview: vk::ImageView,
    sampler: vk::Sampler,
    chars: HashMap<char, Char>,
  ) -> Self {
    Self {
      device,
      mem,
      tex,
      texview,
      sampler,
      chars,
    }
  }

  pub fn get(&self, c: char) -> Char {
    self.chars.get(&c).cloned().unwrap_or(Char {
      tex00: vec2!(0.0),
      tex11: vec2!(0.0),
      size: vec2!(0.0),
      bearing: vec2!(0.0),
      advance: vec2!(0.0),
    })
  }
}
