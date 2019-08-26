use crate::font::*;
use crate::rect::Rect;
use crate::select::SelectId;
use crate::sprites::Text;
use crate::style::event;
use crate::style::Style;
use crate::style::StyleComponent;
use crate::window::Component;
use crate::window::Layout;
use crate::window::Screen;
use crate::ImGui;

#[derive(Debug)]
pub enum Event {
  Unhandled(event::Event),
  Changed,
  Enter,
}

pub trait TextBoxEventHandler: Default {
  type Output: std::fmt::Debug;
  fn handle<S: Style>(_tb: &mut TextBox<S, Self>, _e: Option<event::Event>, _screen: &Screen<S>) -> Option<Self::Output> {
    None
  }

  fn receive_character<S: Style>(tb: &mut TextBox<S, Self>, e: &vk::winit::Event, multiline: bool) -> Option<Event> {
    match e {
      vk::winit::Event::WindowEvent {
        event: vk::winit::WindowEvent::ReceivedCharacter(mut c),
        ..
      } => {
        if !multiline && (c == '\n' || c == '\r') {
          return Some(Event::Enter);
        }

        if c == '\r' {
          c = '\n';
        }

        let ts = tb.get_typeset();
        if let Some(mut cp) = tb.get_cursor() {
          let i = ts.index_of(cp, tb.get_text());
          let mut text = tb.get_text().to_owned();
          text.insert(i, c);
          tb.text(&text);

          if c == '\n' {
            cp.x = 0;
            cp.y += 1;
          } else {
            cp.x += 1;
          }
          tb.cursor(Some(cp));
        }

        Some(Event::Changed)
      }
      _ => None,
    }
  }

  fn move_cursor<S: Style>(tb: &mut TextBox<S, Self>, e: &vk::winit::Event) {
    match e {
      vk::winit::Event::DeviceEvent {
        event:
          vk::winit::DeviceEvent::Key(vk::winit::KeyboardInput {
            state: vk::winit::ElementState::Pressed,
            virtual_keycode: Some(k),
            ..
          }),
        ..
      } => {
        use vk::winit::VirtualKeyCode;
        if let Some(mut c) = tb.get_cursor() {
          match k {
            VirtualKeyCode::Up => c.y = c.y.saturating_sub(1),
            VirtualKeyCode::Down => c.y += 1,
            VirtualKeyCode::Left => c.x = c.x.saturating_sub(1),
            VirtualKeyCode::Right => c.x += 1,
            _ => {}
          }
          tb.cursor(Some(tb.get_typeset().clamp_cursor(c, tb.get_text())));
        }
      }
      _ => {}
    }
  }

  fn set_cursor<S: Style>(tb: &mut TextBox<S, Self>, e: Option<event::Event>) -> Option<Event> {
    if let Some(event::Event::Pressed(event::EventButton { position, .. })) = e {
      let click = vec2!(
        position.x.saturating_sub(tb.text.get_position().x as u32),
        position.y.saturating_sub(tb.text.get_position().y as u32)
      );
      let ts = tb.get_typeset();
      let cp = ts.find_pos(click, tb.get_text());
      tb.cursor(Some(cp));
    }
    e.map(|e| Event::Unhandled(e))
  }
}

pub struct TextBox<S: Style, H: TextBoxEventHandler = HandlerReadonly> {
  text: Text<S>,
  style: S::Component,
  handler: std::marker::PhantomData<H>,
}

impl<S: Style, H: TextBoxEventHandler> TextBox<S, H> {
  pub fn new(gui: &ImGui<S>) -> Self {
    let text = Text::new(gui);
    let style = S::Component::new(gui, "TextBox".to_owned(), false, false);
    Self {
      text,
      style,
      handler: std::marker::PhantomData,
    }
  }

  pub fn get_gui(&self) -> ImGui<S> {
    self.text.get_gui()
  }

  pub fn text(&mut self, text: &str) -> &mut Self {
    self.text.text(text);
    self
  }
  pub fn get_text<'a>(&'a self) -> &'a str {
    self.text.get_text()
  }

  pub fn style(&mut self, style: &str) -> &mut Self {
    self.style.change_style(style, false, false);
    self
  }

  pub fn typeset(&mut self, ts: TypeSet) -> &mut Self {
    self.text.typeset(ts);
    self
  }
  pub fn get_typeset(&self) -> TypeSet {
    self.text.get_typeset()
  }

  pub fn cursor(&mut self, cp: Option<vkm::Vec2u>) -> &mut Self {
    self.text.cursor(cp);
    self
  }
  pub fn get_cursor(&self) -> Option<vkm::Vec2u> {
    self.text.get_cursor()
  }
}

impl<S: Style, H: TextBoxEventHandler> Component<S> for TextBox<S, H> {
  fn rect(&mut self, rect: Rect) -> &mut Self {
    // set the rect of the style first, we get the client area for the textbox from the style
    self.style.rect(rect);
    self.text.position(self.style.get_client_rect().position);
    self
  }
  fn get_rect(&self) -> Rect {
    self.style.get_rect()
  }

  fn get_size_hint(&self) -> vkm::Vec2u {
    // lines() does not count the last empty line, so we check for a trailing linebreak
    let h = (self.get_text().lines().count()
      + match self.get_text().chars().last() {
        Some('\n') => 1,
        _ => 0,
      }) as f32
      * self.get_typeset().line_spacing
      * self.get_typeset().size as f32;
    vec2!(0, self.style.get_padded_size(vec2!(0, h as u32)).y)
  }

  type Event = H::Output;
  fn draw<L: Layout>(&mut self, screen: &mut Screen<S>, layout: &mut L, focus: &mut SelectId) -> Option<H::Output> {
    // style is resized along with the textbox
    let scissor = layout.apply(self);

    // draw and select
    let e = self.style.draw(screen, layout, focus);
    if !self.style.has_focus() {
      self.text.cursor(None);
    }
    screen.push_draw(self.text.get_mesh(), scissor);

    H::handle(self, e, screen)
  }
}

#[derive(Default)]
pub struct HandlerReadonly {}
impl TextBoxEventHandler for HandlerReadonly {
  type Output = event::Event;
  fn handle<S: Style>(_tb: &mut TextBox<S, Self>, e: Option<event::Event>, _screen: &Screen<S>) -> Option<event::Event> {
    e
  }
}

#[derive(Default)]
pub struct HandlerEdit {}
impl TextBoxEventHandler for HandlerEdit {
  type Output = Event;
  fn handle<S: Style>(tb: &mut TextBox<S, Self>, e: Option<event::Event>, screen: &Screen<S>) -> Option<Event> {
    if tb.style.has_focus() {
      for e in screen.get_events() {
        if let Some(e) = Self::receive_character(tb, e, false) {
          return Some(e);
        }
        Self::move_cursor(tb, e);
      }
    }
    Self::set_cursor(tb, e)
  }
}

#[derive(Default)]
pub struct HandlerMultilineEdit {}
impl TextBoxEventHandler for HandlerMultilineEdit {
  type Output = Event;
  fn handle<S: Style>(tb: &mut TextBox<S, Self>, e: Option<event::Event>, screen: &Screen<S>) -> Option<Event> {
    if tb.style.has_focus() {
      for e in screen.get_events() {
        if let Some(e) = Self::receive_character(tb, e, true) {
          return Some(e);
        }
        Self::move_cursor(tb, e);
      }
    }
    Self::set_cursor(tb, e)
  }
}

pub type TextEdit<S> = TextBox<S, HandlerEdit>;
pub type TextEditMultiline<S> = TextBox<S, HandlerMultilineEdit>;
