use super::Component;
use super::Layout;
use super::RootWindow;
use crate::rect::Rect;
use crate::ImGui;
use vk::cmd::commands::Scissor;
use vk::cmd::stream::*;

pub struct Window<T: Layout> {
  root: RootWindow,
  layout: T,
}

impl<T: Layout> Window<T> {
  pub fn new(root: RootWindow, layout: T) -> Self {
    Self { root, layout }
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
    self.layout.push(c);
    self.root.push(c);
    self
  }

  pub fn end_window(self) -> RootWindow {
    self.root
  }
}
