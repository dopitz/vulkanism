extern crate nobs_vulkanism_headless as vk;
#[macro_use]
extern crate nobs_vkmath as vkm;
extern crate nobs_imgui_font_macro as fnt;

use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct FontID {
  name: String,
  size: u32,
}

impl FontID {
  pub fn new(name: &str, size: u32) -> Self {
    Self {
      name: name.to_owned(),
      size,
    }
  }
}

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
    self.mem.trash.push(self.tex);
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

pub trait FontChar {
  fn set_position(&mut self, p: vkm::Vec2f);
  fn set_size(&mut self, s: vkm::Vec2f);
  fn set_tex(&mut self, t00: vkm::Vec2f, t11: vkm::Vec2f);
}

#[derive(Clone)]
pub struct TypeSet {
  pub font: Arc<Font>,
  pub size: u32,
  pub offset: vkm::Vec2i,
  pub cursor: Option<vkm::Vec2u>,
}

impl TypeSet {
  pub fn new(font: Arc<Font>) -> Self {
    Self {
      font,
      size: 12,
      offset: vec2!(0, 12),
      cursor: None,
    }
  }

  pub fn font(mut self, f: Arc<Font>) -> Self {
    self.font = f;
    self
  }

  pub fn size(mut self, s: u32) -> Self {
    self.offset.y -= self.size as i32;
    self.size = s;
    self.offset.y += self.size as i32;
    self
  }

  pub fn offset(mut self, o: vkm::Vec2i) -> Self {
    self.offset = o;
    self.offset.y += self.size as i32;
    self
  }

  pub fn cursor(mut self, c: Option<vkm::Vec2u>) -> Self {
    self.cursor = c;
    self
  }

  pub fn compute<T: FontChar>(&mut self, s: &str, buf: &mut [T]) {
    let size = self.size as f32;
    let offset = self.offset.into();
    let mut off = offset;
    let mut cp = vec2!(0, 0);
    let mut co = vec2!(0.0, 0.0);
    for (c, s) in s.chars().zip(buf.iter_mut()) {
      if c == '\n' || c == '\r' {
        off.x = offset.x;
        off.y = off.y + size;
        cp.x = 0;
        cp.y += 1;
      }

      let ch = self.font.get(c);
      s.set_tex(ch.tex00, ch.tex11);
      s.set_size(ch.size * size);
      s.set_position(off + ch.bearing * size);
      off += ch.advance * size;

      cp.x += 1;
      if let Some(c) = self.cursor {
        if c == cp {
          co = off;
        }
      }
    }

    if self.cursor.is_some() {
      let ch = self.font.get('|');
      let s = &mut buf[s.len()];
      s.set_tex(ch.tex00, ch.tex11);
      s.set_size(ch.size * size);
      s.set_position(co + vec2!(0.0, ch.bearing.y) * size);
    }
  }
}

impl PartialEq for TypeSet {
  fn eq(&self, other: &Self) -> bool {
    Arc::ptr_eq(&self.font, &other.font) && self.size == other.size && self.offset == other.offset && self.cursor == other.cursor
  }
}
impl Eq for TypeSet {}

pub mod dejavu {
  use crate::Char;
  use crate::Font;
  use vk::builder::*;

  fnt::make_font! {
    font = "fonts/DejaVuSans.ttf",
  }
}

pub mod dejavu_mono {
  use crate::Char;
  use crate::Font;
  use vk::builder::*;

  fnt::make_font! {
    font = "fonts/DejaVuSansMono.ttf",
  }
}

pub mod dejavu_serif {
  use crate::Char;
  use crate::Font;
  use vk::builder::*;

  fnt::make_font! {
    font = "fonts/DejaVuSerif.ttf",
  }
}
