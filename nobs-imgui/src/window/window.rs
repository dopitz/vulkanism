use super::ColumnLayout;
use super::Component;
use super::Layout;
use super::Screen;
use crate::rect::Rect;
use crate::style::Style;
use crate::select::SelectId;
use vk::cmd::commands::Scissor;
use vk::pass::MeshId;

/// Window used to set position and size of gui components
///
/// The Window defines a region on the screen on which components are draw
/// It is basically a builder pattern around a [Layout](struct.Layout.html) and [Screen](struct.Streen.html)
pub struct Window<L: Layout> {
  layout: L,
}

impl<L: Layout> Layout for Window<L> {
  fn new(rect: Rect) -> Self {
    Self { layout: L::new(rect) }
  }

  fn get_rect(&self) -> Rect {
    self.layout.get_rect()
  }

  fn apply<S: Style, C: Component<S>>(&mut self, c: &mut C) -> Scissor {
    self.layout.apply(c)
  }
}

impl Default for Window<ColumnLayout> {
  fn default() -> Self {
    Self::new(Default::default())
  }
}

impl<L: Layout> Window<L> {
  pub fn reset(&mut self) {
    self.layout = L::new(self.get_rect());
  }

  pub fn rect(mut self, rect: Rect) -> Self {
    self.layout = L::new(rect);
    self
  }

  /// Sets size and position of the Window in pixel coordinates
  pub fn size(mut self, w: u32, h: u32) -> Self {
    let pos = self.layout.get_rect().position;
    self.rect(Rect::new(pos, vkm::Vec2::new(w, h)))
  }
  /// Sets the position of the Window in pixel coordinates
  pub fn position(mut self, x: i32, y: i32) -> Self {
    let size = self.layout.get_rect().size;
    self.rect(Rect::new(vkm::Vec2::new(x, y), size))
  }
}
