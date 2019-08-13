use crate::font::*;
use crate::sprites;
use crate::style::Style;
use crate::ImGui;

use vk::pass::MeshId;
use vkm::Vec2i;

pub struct Text<S: Style> {
  sprites: sprites::Sprites<S>,

  text: String,
  typeset: TypeSet,
}

impl<S: Style> Text<S> {
  pub fn new(gui: &ImGui<S>) -> Self {
    let sprites = sprites::Sprites::new(gui);
    let typeset = TypeSet::new(gui.get_font());
    Self {
      sprites,
      text: "".to_string(),
      typeset,
    }
  }

  pub fn get_gui(&self) -> ImGui<S> {
    self.sprites.get_gui()
  }

  pub fn get_mesh(&self) -> MeshId {
    self.sprites.get_mesh()
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

    let mut buffer: Vec<sprites::Vertex> = Vec::with_capacity(self.text.len() + 1);
    unsafe { buffer.set_len(self.text.len() + 1) };
    let size = self.typeset.compute(&self.text, &mut buffer);

    self.sprites.sprites(&buffer[0..size]);
  }
}
