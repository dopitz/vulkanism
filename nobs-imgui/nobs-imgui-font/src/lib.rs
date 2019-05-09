extern crate nobs_vulkanism_headless as vk;
#[macro_use]
extern crate nobs_vkmath as vkm;
extern crate nobs_imgui_font_macro as fnt;

use vk::builder::Buildable;
use vk::cmd;
use vk::mem;

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
  pub tex: vk::Image,
  pub texview: vk::ImageView,
  pub sampler: vk::Sampler,

  pub chars: std::collections::HashMap<char, Char>,
  pub char_height: f32,
}

impl Drop for Font {
  fn drop(&mut self) {

  }
}

impl Font {
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

pub struct TypeSet<'a> {
  font: &'a Font,
  size: f32,
  offset: vkm::Vec2f,
}

impl<'a> TypeSet<'a> {
  pub fn new(font: &'a Font) -> Self {
    Self {
      font,
      size: 12.0 * font.char_height,
      offset: vec2!(0.0),
    }
  }

  pub fn size(mut self, s: f32) -> Self {
    self.size = s;
    self
  }

  pub fn offset(mut self, o: vkm::Vec2f) -> Self {
    self.offset = o;
    self
  }

  pub fn compute<T: FontChar>(self, s: &str, buf: &mut [T]) {
    let mut off = self.offset;
    for (c, s) in s.chars().zip(buf.iter_mut()) {
      if c == '\n' || c == '\r' {
        off.x = self.offset.x;
        off.y = off.y + self.size;
      }

      let ch = self.font.get(c);
      s.set_tex(ch.tex00, ch.tex11);
      s.set_size(ch.size * self.size);
      s.set_position(off + ch.bearing * self.size);
      off += ch.advance * self.size;
    }
  }
}


//pub mod dejavu {
//  use crate::font::Char;
//  use crate::font::Font;
//  use crate::ImGui;
//  use vk::builder::*;
//
//  fnt::make_font! {
//    font = "dejavu/DejaVuSans.ttf",
//    margin = 16,
//    char_height = 640,
//    downsample = 16,
//  }
//}

pub mod dejavu_mono {
  use crate::Char;
  use crate::Font;
  use vk::builder::*;

  fnt::make_font! {
    font = "dejavu/DejaVuSansMono.ttf",
    margin = 16,
    char_height = 640,
    downsample = 16,
  }
}
