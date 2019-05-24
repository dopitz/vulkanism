mod pipeline;
use pipeline as pipe;

use crate::cachedpipeline::*;
use crate::font::*;
use crate::sprite;
use crate::window::Window;
use crate::ImGui;

use vk;
use vk::cmd;
use vk::cmd::commands as cmds;
use vkm::Vec2i;

pub struct Text {
  sprites: sprite::Sprites,

  text: String,
  font: std::sync::Arc<Font>,
  font_size: u32,
}

impl Text {
  pub fn new(gui: &ImGui) -> Self {
    let sprites = sprite::Sprites::new(gui);
    let font = gui.get_font();
    let font_size = 20;
    Self {
      sprites,
      text: "".to_string(),
      font,
      font_size,
    }
  }

  pub fn text(&mut self, text: &str) -> &mut Self {
    if self.text != text {
      self.text = text.to_owned();
      self.update_vb();
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

  pub fn font(&mut self, font: std::sync::Arc<Font>, size: u32) -> &mut Self {
    if !std::sync::Arc::ptr_eq(&self.font, &font) {
      self.font = font;
      self.font_size = size;
      self.sprites.texture(self.font.texview, self.font.sampler);
      self.update_vb();
    }
    self
  }

  fn update_vb(&mut self) {
    if self.text.is_empty() {
      return;
    }

    let mut buffer = Vec::with_capacity(self.text.len());
    unsafe { buffer.set_len(self.text.len()) };
    TypeSet::new(&*self.font)
      .offset(vec2!(0, self.font_size).into())
      .size(self.font_size)
      .compute(&self.text, &mut buffer);

    self.sprites.sprites(&buffer);
  }
}

impl cmds::StreamPush for Text {
  fn enqueue(&self, cs: cmd::Stream) -> cmd::Stream {
    cs.push(&self.sprites)
  }
}
