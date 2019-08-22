use super::Component;
use super::FloatLayout;
use super::Layout;
use super::Screen;
use crate::components::TextBox;
use crate::rect::Rect;
use crate::select::SelectId;
use crate::style::event;
use crate::style::Style;
use crate::style::StyleComponent;
use crate::ImGui;
use vk::cmd::commands::Scissor;

/// Window used to set position and size of gui components
///
/// The Window defines a region on the screen on which components are draw
/// It is basically a builder pattern around a [Layout](struct.Layout.html) and [Screen](struct.Streen.html)
pub struct Window<L: Layout, S: Style> {
  padding: vkm::Vec2u,
  layout_window: FloatLayout,
  layout_caption: FloatLayout,
  layout_client: FloatLayout,
  layout: L,

  style: S::Component,
  caption: TextBox<S>,
}

impl<L: Layout, S: Style> Layout for Window<L, S> {
  fn restart(&mut self) {
    self.layout.restart();
  }

  fn set_rect(&mut self, mut rect: Rect) {
    // always show the caption
    let h = self.caption.get_size_hint().y;
    if rect.size.y < h {
      rect.size.y = h
    }

    // Sets the rect of the style and uses the client rect for the layout
    self.layout_window.set_rect(rect);
    self.style.rect(rect);
    let mut cr = self.style.get_client_rect();

    // make room for the window caption
    // use the remainder for the client layout
    self.layout_caption.set_rect(Rect::new(cr.position, cr.size.map_y(|_| h)));
    self.caption.rect(self.layout_caption.get_rect());

    self.layout_client.set_rect(Rect::new(
      cr.position.map_y(|p| p.y + h as i32) + self.padding.into(),
      vec2!(
        cr.size.x.saturating_sub(self.padding.x * 2),
        cr.size.y.saturating_sub(self.padding.y * 2 + h)
      ),
    ));
    self.layout.set_rect(self.layout_client.get_rect());
  }

  fn get_rect(&self) -> Rect {
    self.layout_client.get_rect()
  }

  fn apply<S2: Style, C: Component<S2>>(&mut self, c: &mut C) -> Scissor {
    self.layout.apply(c);
    self.layout_client.get_scissor(c.get_rect())
  }
}

impl<L: Layout, S: Style> Component<S> for Window<L, S> {
  fn rect(&mut self, rect: Rect) -> &mut Self {
    // We delegate to Layout::set_rect, because both function should do the same
    // Layout::set_rect will make room for the caption of the window
    self.set_rect(rect);
    self
  }
  fn get_rect(&self) -> Rect {
    self.layout_client.get_rect()
  }

  fn get_size_hint(&self) -> vkm::Vec2u {
    self.layout_window.get_rect().size
  }

  type Event = ();
  fn draw<LSuper: Layout>(&mut self, screen: &mut Screen<S>, layout: &mut LSuper, focus: &mut SelectId) -> Option<Self::Event> {
    // restart the layout for components that are using this window as layouting scheme
    self.restart();

    // resizes all layouts, caption and style components
    layout.apply(self);

    // draw caption and move window on drag
    if let Some(event::Event::Drag(drag)) = self.caption.draw(screen, &mut self.layout_caption, focus) {
      let mut r = self.layout_window.get_rect();
      r.position += drag.delta;
      self.rect(r);
    }

    // draw the window style
    // resize and move when style body/border is clicked
    if let Some(e) = self.style.draw(screen, &mut self.layout_window, focus) {
      match e {
        event::Event::Resize(rect) => {
          self.rect(rect);
        }
        _ => (),
      }
    }

    None
  }
}

impl<L: Layout, S: Style> Window<L, S> {
  pub fn new(gui: &ImGui<S>, layout: L) -> Self {
    let style = S::Component::new(gui, "Window".to_owned(), true, true);
    let mut caption = TextBox::new(gui);
    caption.text("A fancy window");
    caption.typeset(gui.style.get_typeset().cursor(None));
    caption.style("WindowHeading");

    Self {
      padding: vec2!(4),
      layout_window: Default::default(),
      layout_caption: Default::default(),
      layout_client: Default::default(),
      layout,
      style,
      caption,
    }
  }

  /// Sets size and position of the Window in pixel coordinates
  pub fn size(mut self, w: u32, h: u32) -> Self {
    let pos = self.layout_window.get_rect().position;
    self.set_rect(Rect::new(pos, vkm::Vec2::new(w, h)));
    self
  }
  /// Sets the position of the Window in pixel coordinates
  pub fn position(mut self, x: i32, y: i32) -> Self {
    let size = self.layout_window.get_rect().size;
    self.set_rect(Rect::new(vkm::Vec2::new(x, y), size));
    self
  }

  pub fn padding(mut self, padding: vkm::Vec2u) -> Self {
    self.padding = padding;
    self
  }
}
