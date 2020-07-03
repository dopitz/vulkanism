use crate::font::*;
use crate::sprites;
use crate::style::Style;
use crate::ImGui;

use vk::pass::MeshId;
use vkm::Vec2i;
use vkm::Vec2u;

pub struct Text<S: Style> {
  sprites: sprites::Sprites<S>,

  text: String,
  typeset: TypeSet,
  cursor: Option<Vec2u>,
}

impl<S: Style> Text<S> {
  pub fn new(gui: &ImGui<S>) -> Self {
    let typeset = gui.style.get_typeset_small();
    let mut sprites = sprites::Sprites::new(gui);
    sprites.texture(typeset.font.texview, typeset.font.sampler);
    Self {
      sprites,
      text: "".to_string(),
      typeset,
      cursor: None,
    }
  }

  pub fn get_gui(&self) -> ImGui<S> {
    self.sprites.get_gui()
  }

  pub fn get_mesh(&self) -> MeshId {
    self.sprites.get_mesh()
  }

  pub fn text(&mut self, text: &str) -> &mut Self {
    if self.text != text {
      self.text = text.to_owned();
      self.update_sprites();
    }
    self
  }
  pub fn get_text<'a>(&'a self) -> &'a str {
    &self.text
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
      self.sprites.texture(ts.font.texview, ts.font.sampler);
      self.typeset = ts;
      self.update_sprites();
    }
    self
  }
  pub fn get_typeset(&self) -> TypeSet {
    self.typeset.clone()
  }

  pub fn cursor(&mut self, cp: Option<Vec2u>) -> &mut Self {
    if self.cursor != cp {
      self.cursor = cp;
      self.update_sprites();
    }
    self
  }
  pub fn get_cursor(&self) -> Option<Vec2u> {
    self.cursor
  }

  fn update_sprites(&mut self) {
    if self.text.is_empty() {
      return;
    }

    let mut buffer: Vec<sprites::Vertex> = Vec::with_capacity(self.text.len() + 1);
    unsafe { buffer.set_len(self.text.len() + 1) };
    let size = self.typeset.compute(&self.text, self.cursor, &mut buffer);

    self.sprites.sprites(&buffer[0..size]);
  }
}
