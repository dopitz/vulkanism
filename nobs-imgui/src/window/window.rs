use super::Component;
use super::Layout;
use super::RootWindow;
use crate::rect::Rect;
use crate::ImGui;
use vk::cmd::commands::Scissor;
use vk::cmd::stream::*;

struct WinComp {
  scissor: Scissor,
  mesh: usize,
}

impl From<(Scissor, usize)> for WinComp {
  fn from(p: (Scissor, usize)) -> Self {
    Self { scissor: p.0, mesh: p.1 }
  }
}

pub struct Window<T: Layout> {
  root: RootWindow,
  components: Vec<WinComp>,
  layout: T,
}

impl<T: Layout> Window<T> {
  pub fn new(root: RootWindow, layout: T) -> Self {
    Self {
      root,
      components: Default::default(),
      layout,
    }
  }

  pub fn rect(mut self, rect: Rect) -> Self {
    self.layout.reset(rect);
    self
  }
  pub fn size(mut self, w: u32, h: u32) -> Self {
    self.layout.reset(Rect::new(self.layout.get_rect().position, vkm::Vec2::new(w, h)));
    self
  }
  pub fn position(mut self, x: i32, y: i32) -> Self {
    self.layout.reset(Rect::new(vkm::Vec2::new(x, y), self.layout.get_rect().size));
    self
  }

  pub fn push<C: Component>(mut self, c: &mut C) -> Self {
    self.components.push(self.layout.push(c).into());
    self.root.push(c);
    self
  }

  pub fn end_window(self) -> RootWindow {
    // TODO: layout, push, ...
    self.root
  }
}
