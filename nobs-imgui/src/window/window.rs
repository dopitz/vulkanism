use super::Component;
use super::Layout;
use super::Screen;
use crate::rect::Rect;
use crate::select::SelectId;
use crate::style::event;
use crate::style::Style;
use crate::style::StyleComponent;
use crate::textbox::TextBox;
use crate::ImGui;
use vk::cmd::commands::Scissor;

/// Window used to set position and size of gui components
///
/// The Window defines a region on the screen on which components are draw
/// It is basically a builder pattern around a [Layout](struct.Layout.html) and [Screen](struct.Streen.html)
pub struct Window<L: Layout, S: Style> {
  layout: L,
  style: Option<S::Component>,
  heading: Option<TextBox<S>>,
}

impl<L: Layout, S: Style> Default for Window<L, S> {
  fn default() -> Self {
    Self {
      layout: Default::default(),
      style: None,
      heading: None,
    }
  }
}

impl<L: Layout, S: Style> Layout for Window<L, S> {
  fn restart(&mut self) {
    self.layout.restart();
  }

  fn set_rect(&mut self, rect: Rect) {
    // Sets the rect of the style and uses the client rect for the layout
    if let Some(style) = self.style.as_mut() {
      style.rect(rect);
      let mut cr = style.get_client_rect();
      // offset the actual drawing plane for components of the window to the client rect of the style with a padding from top for the heading
      let head = self.heading.as_ref().unwrap().get_typeset().size;
      cr.position.y += head as i32;
      cr.size.y = cr.size.y.saturating_sub(head);
      self.layout.set_rect(cr);
    } else {
      self.layout.set_rect(rect);
    }
  }

  fn get_rect(&self) -> Rect {
    // Always return the layout's rect here!
    // This is important in case we want to do things with the layout's draw area (e.g. Layout::get_scissor)
    self.layout.get_rect()
  }

  fn apply<S2: Style, C: Component<S2>>(&mut self, c: &mut C) -> Scissor {
    self.layout.apply(c)
  }
}

impl<L: Layout, S: Style> Component<S> for Window<L, S> {
  fn rect(&mut self, rect: Rect) -> &mut Self {
    // We delegate to Layout::set_rect, because both function should do the same
    self.set_rect(rect);
    self
  }
  fn get_rect(&self) -> Rect {
    // Return the the style's rect here, because we are accassing the window as a component
    if let Some(style) = self.style.as_ref() {
      style.get_rect()
    } else {
      self.layout.get_rect()
    }
  }

  fn get_size_hint(&self) -> vkm::Vec2u {
    if let Some(style) = self.style.as_ref() {
      style.get_size_hint()
    } else {
      self.layout.get_rect().size
    }
  }

  type Event = ();
  fn draw<LSuper: Layout>(&mut self, screen: &mut Screen<S>, layout: &mut LSuper, focus: &mut SelectId) -> Option<Self::Event> {
    // restart the layout for components that are using this window as layouting scheme
    self.restart();
    let scissor = layout.apply(self);

    // window heading needs rect for moving
    let mut r = Component::get_rect(self);
    if let Some(heading) = self.heading.as_mut() {
      // rect for the window heading
      let mut cr = if let Some(style) = self.style.as_ref() {
        style.get_client_rect()
      } else {
        self.layout.get_rect()
      };
      cr.size.y = heading.get_typeset().size;
      // draw caption and move window on drag
      if let Some(event::Event::Drag(drag)) = heading.draw(screen, &mut super::ColumnLayout::from(cr), focus) {
        r.position += drag.delta;
        self.rect(r);
      }
    }

    // draw the window style
    let e = if let Some(style) = self.style.as_mut() {
      style.draw(screen, layout, focus)
    } else {
      None
    };

    // resize and move when style body/border is clicked
    if let Some(e) = e {
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
  pub fn new(gui: &ImGui<S>) -> Self {
    let style = Some(S::Component::new(gui, "Window".to_owned(), true, true));
    let mut heading = TextBox::new(gui);
    heading.text("A fancy window");
    heading.typeset(heading.get_typeset().size(32).cursor(None));
    heading.style("WindowHeading");
    let heading = Some(heading);

    Self {
      layout: Default::default(),
      style,
      heading,
    }
  }

  /// Sets size and position of the Window in pixel coordinates
  pub fn size(mut self, w: u32, h: u32) -> Self {
    let pos = self.layout.get_rect().position;
    self.set_rect(Rect::new(pos, vkm::Vec2::new(w, h)));
    self
  }
  /// Sets the position of the Window in pixel coordinates
  pub fn position(mut self, x: i32, y: i32) -> Self {
    let size = self.layout.get_rect().size;
    self.set_rect(Rect::new(vkm::Vec2::new(x, y), size));
    self
  }
}
