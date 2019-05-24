use crate::font::*;
use crate::rect::Rect;
use crate::text::Text;
use crate::window::Window;
use crate::ImGui;

use vk;
use vk::cmd;
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

  pub fn font(&mut self, font: std::sync::Arc<Font>, size: u32) -> &mut Self {
    self.text.font(font, size);
    self
  }

  pub fn rect(&mut self, rect: Rect) -> &mut Self {
    if Rect::from_vkrect(self.rect.rect) != rect {
      self.text.position(rect.position);
      self.rect.rect = rect.to_vkrect();
    }
    self
  }

  pub fn get_text(&self) -> String {
    self.text.get_text()
  }
}

impl cmds::StreamPush for TextBox {
  fn enqueue(&self, cs: cmd::Stream) -> cmd::Stream {
    cs.push(&self.rect).push(&self.text)
  }
}

impl crate::window::Component for TextBox {
  fn add_compontent(&mut self, wnd: &mut Window) {
    self.rect(wnd.get_next_bounds());
  }
}
