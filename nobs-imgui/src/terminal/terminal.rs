use crate::style::Style;
use crate::window::*;
use crate::ImGui;
use crate::select::SelectId;
use crate::rect::Rect;

pub struct Terminal<S: Style> {
  wnd: Window<ColumnLayout, S>,
}

impl<S: Style> Size for Terminal<S> {
  fn rect(&mut self, rect: Rect) -> &mut Self {
    self.wnd.rect(rect);
    self
  }

  fn get_rect(&self) -> Rect {
    self.wnd.get_rect()
  }

  fn get_size_hint(&self) -> vkm::Vec2u {
    self.wnd.get_size_hint()
  }
}

impl<S: Style> Component<S> for Terminal<S> {
  type Event = ();
  fn draw<L: Layout>(&mut self, screen: &mut Screen<S>, layout: &mut L, focus: &mut SelectId) -> Option<Self::Event> {
    self.wnd.draw(screen, layout, focus);
    None
  }
}

impl<S: Style> Terminal<S> {
  pub fn new(gui: &ImGui<S>) -> Self {
    let mut wnd = Window::new(&gui, ColumnLayout::default());
    wnd
      .caption("terminal")
      .position(20, 20)
      .size(500, 420)
      .focus(true)
      .draw_caption(false);
    Self { wnd }
  }
}
