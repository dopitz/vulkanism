use crate::font::*;
use crate::rect::Rect;
use crate::text::Text;
use crate::window::Component;
use crate::ImGui;

pub struct TextBox {
  rect: Rect,
  text: Text,
}

impl TextBox {
  pub fn new(gui: &ImGui) -> Self {
    let rect = Rect::from_rect(0, 0, 200, 20);
    let text = Text::new(gui);

    Self { rect, text }
  }

  pub fn text(&mut self, text: &str) -> &mut Self {
    self.text.text(text);
    self
  }
  pub fn get_text(&self) -> String {
    self.text.get_text()
  }

  pub fn typeset(&mut self, ts: TypeSet) -> &mut Self {
    self.text.typeset(ts);
    self
  }
  pub fn get_typeset(&self) -> TypeSet {
    self.text.get_typeset()
  }
}

impl Component for TextBox {
  fn rect(&mut self, rect: Rect) -> &mut Self {
    if self.rect != rect {
      self.text.position(rect.position);
      self.rect = rect;
    }
    self
  }
  fn get_rect(&self) -> Rect {
    self.rect
  }

  fn get_size_hint(&self) -> vkm::Vec2u {
    let w = 0;
    let h = self.get_text().lines().count() as f32 * self.get_typeset().line_spacing * self.get_typeset().size as f32;
    vec2!(w, h as u32)
  }

  fn get_mesh(&self) -> usize {
    self.text.get_mesh()
  }
}
