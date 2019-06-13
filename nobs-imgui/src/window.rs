use crate::rect::Rect;
use crate::ImGui;
use vk::cmd::commands::Scissor;
use vk::cmd::stream::*;

pub trait Component {
  fn rect(&mut self, rect: Rect) -> &mut Self;
  fn get_rect(&self) -> Rect;

  fn get_size_hint(&self) -> vkm::Vec2u;

  fn get_mesh(&self) -> usize;
}

pub struct WinComp {
  scissor: Scissor,
  mesh: usize,
}

pub trait Layout {
  fn push<T: Component>(&mut self, c: &mut T) -> WinComp;
}

pub struct ColumnLayout {
  pub rect: Rect,
  pub top: u32,
}

impl Layout for ColumnLayout {
  fn push<T: Component>(&mut self, c: &mut T) -> WinComp {
    let mut rect = Rect::new(self.rect.position + vec2!(0, self.top as i32), c.get_size_hint());
    if rect.size.x == 0 {
      rect.size.x = self.rect.size.x;
    }
    if rect.size.x >= self.rect.size.x {
      rect.size.x = self.rect.size.x;
    }
    if self.top + rect.size.y >= self.rect.size.y {
      rect.size.y = self.rect.size.y - self.top;
    }
    c.rect(rect);

    WinComp {
      scissor: Scissor::with_rect(rect.to_vkrect()),
      mesh: c.get_mesh(),
    }
  }
}

pub struct Window {
  gui: ImGui,
  components: Vec<WinComp>,
  layout: ColumnLayout,
}

impl Window {
  pub fn new(gui: ImGui) -> Self {
    Self {
      gui,
      components: Default::default(),
      layout: ColumnLayout {
        rect: Default::default(),
        top: 0,
      },
    }
  }

  pub fn rect(mut self, rect: Rect) -> Self {
    self.layout.rect = rect;
    self
  }
  pub fn size(mut self, w: u32, h: u32) -> Self {
    self.layout.rect.size = vkm::Vec2::new(w, h);
    self
  }
  pub fn position(mut self, x: i32, y: i32) -> Self {
    self.layout.rect.position = vkm::Vec2::new(x, y);
    self
  }

  pub fn push<T: Component>(mut self, c: &mut T) -> Self {
    self.components.push(self.layout.push(c));
    self
  }
}

impl StreamPush for Window {
  fn enqueue(&self, cs: CmdBuffer) -> CmdBuffer {
    let mut cs = self.gui.begin(cs).push(&Scissor::with_rect(self.layout.rect.to_vkrect()));

    let meshes = self.gui.get_meshes();
    for c in self.components.iter() {
      cs = cs.push(&c.scissor).push(&meshes.get(c.mesh));
    }

    self.gui.end(cs)
  }
}
