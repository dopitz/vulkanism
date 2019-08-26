use crate::rect::Rect;
use crate::select::SelectId;
use crate::style::Style;
use crate::window::Component;
use crate::window::Layout;
use crate::window::Screen;
use crate::window::Size;

pub struct Spacer {
  rect: Rect,
}

impl Spacer {
  pub fn new(size: vkm::Vec2u) -> Self {
    Self {
      rect: Rect::new(vec2!(0), size),
    }
  }
}

impl Size for Spacer {
  fn rect(&mut self, rect: Rect) -> &mut Self {
    self.rect = rect;
    self
  }
  fn get_rect(&self) -> Rect {
    self.rect
  }

  fn get_size_hint(&self) -> vkm::Vec2u {
    self.rect.size
  }
}

impl<S: Style> Component<S> for Spacer {
  type Event = ();
  fn draw<L: Layout>(&mut self, _screen: &mut Screen<S>, layout: &mut L, _focus: &mut SelectId) -> Option<()> {
    // just apply the layout, so that it can advance the spacing for following components
    layout.apply::<S, Self>(self);
    None
  }
}

impl From<vkm::Vec2u> for Spacer {
  fn from(size: vkm::Vec2u) -> Self {
    Self::new(size)
  }
}

impl From<Rect> for Spacer {
  fn from(rect: Rect) -> Self {
    Self { rect }
  }
}
