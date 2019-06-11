use crate::font::*;
use crate::rect::Rect;
use crate::text::Text;
use crate::window::Component;
use crate::ImGui;

use vk;
use vk::cmd::commands as cmds;

pub struct TextBox {
  rect: cmds::Scissor,
  text: Text,
}

impl TextBox {
  pub fn new(gui: &ImGui) -> Self {
    let rect = cmds::Scissor::with_size(200, 20);
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

  pub fn rect(&mut self, rect: Rect) -> &mut Self {
    if Rect::from_vkrect(self.rect.rect) != rect {
      self.text.position(rect.position);
      self.rect.rect = rect.to_vkrect();
    }
    self
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
  fn rect(&mut self, rect: Rect) {
    if Rect::from_vkrect(self.rect.rect) != rect {
      self.text.position(rect.position);
      self.rect.rect = rect.to_vkrect();
    }
  }

  fn get_mesh(&self) -> usize {
    self.text.get_mesh()
  }
}
