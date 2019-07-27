use super::Font;
use std::sync::Arc;
use vkm;

pub trait FontChar {
  fn set_position(&mut self, p: vkm::Vec2f);
  fn set_size(&mut self, s: vkm::Vec2f);
  fn set_tex(&mut self, t00: vkm::Vec2f, t11: vkm::Vec2f);
}

#[derive(Clone)]
pub struct TypeSet {
  pub font: Arc<Font>,
  pub size: u32,
  pub line_spacing: f32,
  pub offset: vkm::Vec2i,
  pub cursor: Option<vkm::Vec2u>,
}

impl TypeSet {
  pub fn new(font: Arc<Font>) -> Self {
    Self {
      font,
      size: 12,
      line_spacing: 1.2,
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

  pub fn line_spacing(mut self, s: f32) -> Self {
    self.line_spacing = s;
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

  pub fn compute<T: FontChar>(&mut self, s: &str, buf: &mut [T]) -> usize {
    let size = self.size as f32;
    let offset = self.offset.into();
    let mut off = offset;
    let mut cp = vec2!(0, 0);
    let mut co = vec2!(0.0, 0.0);
    for (c, s) in s.chars().zip(buf.iter_mut()) {
      let ch = self.font.get(c);
      s.set_tex(ch.tex00, ch.tex11);
      s.set_size(ch.size * size);
      s.set_position(off + ch.bearing * size);
      off += ch.advance * size;

      cp.x += 1;
      if let Some(c) = self.cursor {
        if cp.y <= c.y && cp.x <= c.x {
          co = off;
        }
      }

      if c == '\n' || c == '\r' {
        off.x = offset.x;
        off.y = off.y + size * self.line_spacing;
        cp.x = 0;
        cp.y += 1;
      }
    }

    if self.cursor.is_some() {
      let ch = self.font.get('|');
      let s = &mut buf[s.len()];
      s.set_tex(ch.tex00, ch.tex11);
      s.set_size(ch.size * size);
      s.set_position(co + vec2!(0.0, ch.bearing.y) * size);

      buf.len()
    } else {
      buf.len() - 1
    }
  }
}

impl PartialEq for TypeSet {
  fn eq(&self, other: &Self) -> bool {
    Arc::ptr_eq(&self.font, &other.font) && self.size == other.size && self.offset == other.offset && self.cursor == other.cursor
  }
}
impl Eq for TypeSet {}
