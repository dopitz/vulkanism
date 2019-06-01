use crate::font::*;
use crate::sprite;
use crate::ImGui;

use vk;
use vk::cmd;
use vk::cmd::commands as cmds;
use vkm::Vec2i;

pub struct Text {
  sprites: sprite::Sprites,

  text: String,
  typeset: TypeSet,
}

impl Text {
  pub fn new(gui: &ImGui) -> Self {
    let sprites = sprite::Sprites::new(gui);
    let typeset = TypeSet::new(gui.get_font());
    Self {
      sprites,
      text: "".to_string(),
      typeset,
    }
  }

  pub fn typesetter(&mut self, ts: TypeSet) -> &mut Self {
    self.typeset = ts;
    self
  }

  pub fn text(&mut self, text: &str) -> &mut Self {
    if self.text != text {
      self.text = text.to_owned();
      self.update_sprites();
    }
    self
  }
  pub fn get_text(&self) -> String {
    self.text.clone()
  }

  pub fn position(&mut self, pos: Vec2i) -> &mut Self {
    self.sprites.position(pos);
    self
  }
  pub fn get_position(&self) -> Vec2i {
    self.sprites.get_position()
  }

  pub fn typeset(&mut self, ts: TypeSet) -> &mut Self {
    if self.typeset != ts {
      self.typeset = ts;
      self.update_sprites();
    }
    self
  }
  pub fn get_typeset(&self) -> TypeSet {
    self.typeset.clone()
  }

  fn update_sprites(&mut self) {
    if self.text.is_empty() {
      return;
    }

    let mut buffer: Vec<sprite::Vertex> = Vec::with_capacity(self.text.len() + 1);
    unsafe { buffer.set_len(self.text.len() + 1) };
    let size = self.typeset.compute(&self.text, &mut buffer);

    self.sprites.sprites(&buffer[0..size]);
  }
}

impl cmds::StreamPush for Text {
  fn enqueue(&self, cs: cmd::Stream) -> cmd::Stream {
    cs.push(&self.sprites)
  }
}
