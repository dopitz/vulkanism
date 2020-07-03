use super::Layout;
use crate::component::textbox::Event as TBEvent;
use crate::component::Component;
use crate::component::Size;
use crate::component::Stream;
use crate::component::TextBox;
use crate::rect::Rect;
use crate::style::event::Event as StyleEvent;
use crate::style::Style;
use crate::style::StyleComponent;
use crate::ImGui;
use vk::cmd::commands::Scissor;

#[derive(Debug, Clone, Copy)]
pub enum Event {
  Resized(Rect),
  Clicked,
  Scroll,
}

/// Window used to set position and size of gui components
///
/// The Window defines a region on the screen on which components are draw
/// It is basically a builder pattern around a [Layout](struct.Layout.html) and [Screen](struct.Streen.html)
pub struct Window<L: Layout + Clone + 'static + 'static, S: Style> {
  layout: WindowLayout<L>,

  style: S::Component,
  caption: TextBox<S>,
  draw_caption: bool,
}

pub struct WindowBegin<'a, L: Layout + Clone + 'static, S: Style> {
  wnd: &'a mut Window<L, S>,
}

impl<'a, L: Layout + Clone + 'static, S: Style> Size for WindowBegin<'a, L, S> {
  fn set_rect(&mut self, rect: Rect) {
    self.wnd.set_rect(rect);
  }
  fn get_rect(&self) -> Rect {
    self.wnd.get_rect()
  }

  fn get_size_hint(&self) -> vkm::Vec2u {
    self.wnd.get_size_hint()
  }
}

impl<'a, L: Layout + Clone + 'static, S: Style> Component<S> for WindowBegin<'a, L, S> {
  type Event = Event;

  fn enqueue<'b, R: std::fmt::Debug>(&mut self, mut s: Stream<'b, S, R>) -> Stream<'b, S, Self::Event> {
    // resize the window itself, it may be nested inside another window
    s.layout(self);

    // draw window background + border and caption
    let s = s.push(&mut self.wnd.style);
    let s = if self.wnd.draw_caption {
      let s = s.push(&mut self.wnd.caption);
      let r = match s.get_result().cloned() {
        Some(TBEvent::Base(e)) => Some(e),
        _ => None,
      };
      s.with_result(r)
    } else {
      s
    };

    // handle resize and drag events from window border/body/caption
    let mut s = match s.get_result().cloned() {
      Some(StyleEvent::Resize(rect)) => {
        self.set_rect(rect);
        s.with_result(Some(Event::Resized(rect)))
      }

      Some(StyleEvent::Drag(drag)) => {
        let mut rect = self.wnd.layout.get_rect();
        rect.position = drag.end.into() - drag.start.relative_pos.into();
        self.set_rect(rect);
        s.with_result(Some(Event::Resized(rect)))
      }

      _ => s.with_result(None),
    };

    // Scrolling with mouse wheel
    if self.wnd.style.has_focus() {
      match s.get_event() {
        Some(vk::winit::event::Event::DeviceEvent {
          event: vk::winit::event::DeviceEvent::Motion { axis: 3, value },
          ..
        }) => self.wnd.layout.scroll(vec2!(0.0, *value as f32)),
        _ => (),
      }
    }

    // restart the layout for components that are using this window as layouting scheme
    self.wnd.layout.restart();
    s.push_layout(Box::new(self.wnd.layout.clone()));

    s.with_result(None)
  }
}

pub struct WindowEnd<'a, L: Layout + Clone + 'static, S: Style> {
  wnd: &'a mut Window<L, S>,
}

impl<'a, L: Layout + Clone + 'static, S: Style> Size for WindowEnd<'a, L, S> {
  fn set_rect(&mut self, rect: Rect) {
    self.wnd.set_rect(rect);
  }
  fn get_rect(&self) -> Rect {
    self.wnd.get_rect()
  }

  fn get_size_hint(&self) -> vkm::Vec2u {
    self.wnd.get_size_hint()
  }
}

impl<'a, L: Layout + Clone + 'static, S: Style> Component<S> for WindowEnd<'a, L, S> {
  type Event = Event;

  fn enqueue<'b, R: std::fmt::Debug>(&mut self, mut s: Stream<'b, S, R>) -> Stream<'b, S, Self::Event> {
    unsafe {
      let raw = Box::into_raw(s.pop_layout()) as *mut WindowLayout<L>;
      self.wnd.layout = (*raw).clone();
      Box::from_raw(raw);
    }
    s.with_result(None)
  }
}

impl<L: Layout + Clone + 'static, S: Style> Size for Window<L, S> {
  fn set_rect(&mut self, rect: Rect) {
    self.layout.caption.size.y = if self.draw_caption { self.caption.get_size_hint().y } else { 0 };
    self.layout.set_rect(rect);
    self.style.set_rect(self.layout.outer);
    self.caption.set_rect(self.layout.caption);
  }
  fn get_rect(&self) -> Rect {
    self.layout.get_rect()
  }

  fn get_size_hint(&self) -> vkm::Vec2u {
    self.layout.get_rect().size
  }
}

impl<L: Layout + Clone + 'static, S: Style> Window<L, S> {
  pub fn new(gui: &ImGui<S>, layout: L) -> Self {
    let style = S::Component::new(gui, "Window".to_owned(), true, true);
    let mut caption = TextBox::new(gui);
    caption.text("A fancy window");
    caption.style("WindowCaption");

    Self {
      layout: WindowLayout::new(layout),
      style,
      caption,
      draw_caption: true,
    }
  }

  pub fn get_client_rect(&self) -> Rect {
    self.layout.client
  }

  pub fn style(&mut self, style: &str, moveable: bool, resizeable: bool) -> &mut Self {
    self.style.change_style(style, moveable, resizeable);
    self
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
    let pos = self.layout.get_rect().position;
    self.set_rect(Rect::new(pos, vkm::Vec2::new(w, h)));
    self
  }
  /// Sets the position of the Window in pixel coordinates
  pub fn position(&mut self, x: i32, y: i32) -> &mut Self {
    let size = self.layout.get_rect().size;
    self.set_rect(Rect::new(vkm::Vec2::new(x, y), size));
    self
  }
  /// Sets padding of components from the (inner) window border
  pub fn padding(&mut self, padding: vkm::Vec2u) -> &mut Self {
    self.layout.padding = padding;
    self
  }

  pub fn scroll(&mut self, scroll: vkm::Vec2u) -> &mut Self {
    //self.layout_scroll = scroll;
    self
  }
  pub fn get_scroll(&self) -> vkm::Vec2u {
    vec2!(0)
    //self.layout_scroll
  }

  pub fn begin<'a>(&'a mut self) -> WindowBegin<'a, L, S> {
    WindowBegin::<'a, L, S> { wnd: self }
  }
  pub fn end<'a>(&'a mut self) -> WindowEnd<'a, L, S> {
    WindowEnd::<'a, L, S> { wnd: self }
  }
}

#[derive(Clone)]
pub struct WindowLayout<L: Layout + Clone + 'static> {
  padding: vkm::Vec2u,
  outer: Rect,
  caption: Rect,
  client: Rect,
  layout: L,

  scroll_pos: vkm::Vec2i,
  scroll_speed: f32,
}

impl<L: Layout + Clone + 'static> Size for WindowLayout<L> {
  fn set_rect(&mut self, rect: Rect) {
    // outer is by definition the desired rect
    self.outer = rect;

    // caption starts at the same position as outer and has the same width
    self.caption.position = self.outer.position;
    self.caption.size.x = self.outer.size.x;

    // client starts below caption and crops the client layout rect
    self.client = rect;
    self.client.position.y = self.client.position.y + self.caption.size.y as i32;
    self.client.size.x = self.client.size.x.saturating_sub(self.padding.x);
    self.client.size.y = self.client.size.y.saturating_sub(self.padding.y + self.caption.size.y);

    // the client layout rect is sized by its contents and shifted by scroll_pos
    let mut layout_rect = self.client;
    layout_rect.size = self.layout.get_size_hint();
    if layout_rect.size.x == 0 {
      layout_rect.size.x = self.client.size.x;
    }
    if layout_rect.size.y == 0 {
      layout_rect.size.y = self.client.size.y;
    }
    layout_rect.position = layout_rect.position + self.scroll_pos;
    self.layout.set_rect(layout_rect);
  }

  fn get_rect(&self) -> Rect {
    self.outer
  }

  fn get_size_hint(&self) -> vkm::Vec2u {
    self.layout.get_size_hint()
  }
}

impl<L: Layout + Clone + 'static> Layout for WindowLayout<L> {
  fn restart(&mut self) {
    self.layout.restart();
  }

  fn layout(&mut self, c: &mut dyn Size) -> Scissor {
    // we compute the size with the client layout
    // but we still scissor with the windows outer rect
    self.layout.layout(c);
    self.get_scissor(c.get_rect())
  }

  fn get_scissor(&self, mut rect: Rect) -> Scissor {
    let lo = vkm::Vec2::clamp(self.client.position, vec2!(0), vec2!(i32::max_value()));
    let hi = self.client.position + self.client.size.into();
    rect.size = (vkm::Vec2::clamp(rect.position + rect.size.into(), lo, hi) - rect.position).into();
    rect.position = vkm::Vec2::clamp(rect.position, lo, hi);
    Scissor::with_rect(rect.into())
  }
}

impl<L: Layout + Clone + 'static> WindowLayout<L> {
  pub fn new(layout: L) -> Self {
    Self {
      caption: Rect::default(),
      padding: vec2!(0),
      outer: Rect::default(),
      client: Rect::default(),
      layout,

      scroll_pos: vec2!(0),
      scroll_speed: 10.0,
    }
  }

  pub fn scroll(&mut self, d: vkm::Vec2f) {
    // TODO fix bug
    // clamp scroll position
    self.scroll_pos = self.scroll_pos + (d * self.scroll_speed).into();
  }
}
