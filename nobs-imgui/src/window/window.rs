use super::Component;
use super::FloatLayout;
use super::Layout;
use super::Screen;
use super::Size;
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
  layout_size: vkm::Vec2u,
  layout_scroll: vkm::Vec2u,

  style: S::Component,
  caption: TextBox<S>,
  draw_caption: bool,
}

impl<L: Layout, S: Style> Size for Window<L, S> {
  fn rect(&mut self, mut rect: Rect) -> &mut Self {
    // reserve space for the caption (if enabled)
    let h = match self.draw_caption {
      true => self.caption.get_size_hint().y,
      false => 0,
    };
    if rect.size.y < h {
      rect.size.y = h
    }

    // the whole window and stile have the same dimensions
    // client and caption use the client area of the style
    self.layout_window.rect(rect);
    self.style.rect(rect);
    let cr = self.style.get_client_rect();

    // make room for the window caption
    // use the remainder for the client layout
    self.layout_caption.rect(Rect::new(cr.position, cr.size.map_y(|_| h)));
    self.caption.rect(self.layout_caption.get_rect());

    let client_rect = Rect::new(
      cr.position.map_y(|p| p.y + h as i32) + self.padding.into(),
      vec2!(
        cr.size.x.saturating_sub(self.padding.x * 2),
        cr.size.y.saturating_sub(self.padding.y * 2 + h)
      ),
    );
    self.layout_client.rect(client_rect);

    // Set client layout with scrolling
    let p0 = client_rect.position;
    let p1 = p0
      - vec2!(
        self.layout_size.x.saturating_sub(client_rect.size.x),
        self.layout_size.y.saturating_sub(client_rect.size.y)
      )
      .into();
    let p = vkm::Vec2::clamp(p0 - self.layout_scroll.into(), p1, p0);

    if self.layout_size.x == 0 {
      self.layout_size.x = client_rect.size.x
    }
    if self.layout_size.y == 0 {
      self.layout_size.y = client_rect.size.y
    }

    self.layout.rect(Rect::new(p, self.layout_size));
    self
  }
  fn get_rect(&self) -> Rect {
    self.layout_window.get_rect()
  }

  fn get_size_hint(&self) -> vkm::Vec2u {
    // TODO: compute actual size from caption and client
    self.layout_window.get_rect().size
  }
}

impl<L: Layout, S: Style> Layout for Window<L, S> {
  fn restart(&mut self) {
    self.layout_size = self.layout.get_size_hint();
    self.layout.restart();
  }

  fn apply<S2: Style, C: Component<S2>>(&mut self, c: &mut C) -> Scissor {
    self.layout.apply(c);
    self.layout_client.get_scissor(c.get_rect())
  }

  fn get_scissor(&self, rect: Rect) -> Scissor {
    self.layout_client.get_scissor(rect)
  }
}

impl<L: Layout, S: Style> Component<S> for Window<L, S> {
  type Event = ();
  fn draw<LSuper: Layout>(&mut self, screen: &mut Screen<S>, layout: &mut LSuper, focus: &mut SelectId) -> Option<Self::Event> {
    // restart the layout for components that are using this window as layouting scheme
    self.restart();

    // resizes all layouts, caption and style components
    layout.apply(self);

    // draw caption and move window on drag
    if self.draw_caption {
      if let Some(event::Event::Drag(drag)) = self.caption.draw(screen, &mut self.layout_caption, focus) {
        let mut r = self.layout_window.get_rect();
        r.position += drag.delta;
        self.rect(r);
      }
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

    // Scrolling with mouse wheel
    if self.style.has_focus() {
      for e in screen.get_events() {
        match e {
          vk::winit::Event::DeviceEvent {
            event: vk::winit::DeviceEvent::Motion { axis: 3, value },
            ..
          } => {
            let value = if *value > 0.0 {
              -15
            } else if *value < 0.0 {
              15
            } else {
              0
            };
            let max = vec2!(
              self.layout_size.x.saturating_sub(self.layout_client.get_rect().size.x),
              self.layout_size.y.saturating_sub(self.layout_client.get_rect().size.y)
            )
            .into();
            self.layout_scroll = vkm::Vec2::clamp(self.layout_scroll.into().map_y(|v| v.y + value), vec2!(0), max).into();
          }
          _ => (),
        }
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
    caption.style("WindowCaption");

    Self {
      padding: vec2!(0),
      layout_window: Default::default(),
      layout_caption: Default::default(),
      layout_client: Default::default(),
      layout,
      layout_size: Default::default(),
      layout_scroll: Default::default(),
      style,
      caption,
      draw_caption: true,
    }
  }

  pub fn focus(&mut self, focus: bool) -> &mut Self {
    self.style.focus(focus);
    self
  }
  pub fn has_focus(&self) -> bool {
    self.style.has_focus()
  }

  /// Sets the caption of the Window
  pub fn caption(&mut self, caption: &str) -> &mut Self {
    self.caption.text(caption);
    self
  }
  /// Sets a flag to enable/disable the caption of the window
  pub fn draw_caption(&mut self, draw_caption: bool) -> &mut Self {
    self.draw_caption = draw_caption;
    self
  }
  /// Sets size and position of the Window in pixel coordinates
  pub fn size(&mut self, w: u32, h: u32) -> &mut Self {
    let pos = self.layout_window.get_rect().position;
    self.rect(Rect::new(pos, vkm::Vec2::new(w, h)));
    self
  }
  /// Sets the position of the Window in pixel coordinates
  pub fn position(&mut self, x: i32, y: i32) -> &mut Self {
    let size = self.layout_window.get_rect().size;
    self.rect(Rect::new(vkm::Vec2::new(x, y), size));
    self
  }
  /// Sets padding of components from the (inner) window border
  pub fn padding(&mut self, padding: vkm::Vec2u) -> &mut Self {
    self.padding = padding;
    self
  }
}
