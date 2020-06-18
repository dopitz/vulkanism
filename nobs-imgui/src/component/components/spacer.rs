use crate::component::Component;
use crate::component::Size;
use crate::component::Stream;
use crate::rect::Rect;
use crate::style::Style;
use crate::window::Layout;

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
  fn set_rect(&mut self, rect: Rect) {
    self.rect = rect;
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
  //fn draw<L: Layout>(
  //  &mut self,
  //  _screen: &mut Screen<S>,
  //  layout: &mut L,
  //  _focus: &mut SelectId,
  //  e: Option<&winit::event::Event<i32>>,
  //) -> Option<()> {
  //  // just apply the layout, so that it can advance the spacing for following components
  //  // only apply the layout, when we do not process event but render compontent
  //  if e.is_none() {
  //    layout.apply::<S, Self>(self);
  //  }
  //  None
  //}

  fn enqueue<'a, R: std::fmt::Debug>(&mut self, mut s: Stream<'a, S, R>) -> Stream<'a, S, Self::Event> {
    s.layout(self);
    s.with_result(None)
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
