use crate::font::*;
use crate::rect::Rect;
use crate::select::SelectId;
use crate::style::event;
use crate::style::Style;
use crate::style::StyleComponent;
use crate::sprites::Text;
use crate::window::Component;
use crate::window::Layout;
use crate::window::Screen;
use crate::ImGui;

#[derive(Debug)]
pub enum Event {
  Clicked,
  Changed,
}

pub trait TextBoxEventHandler: Default {
  type Output: std::fmt::Debug;
  fn handle<S: Style>(tb: &mut TextBox<S, Self>, e: Option<event::Event>, screen: &Screen<S>) -> Option<Self::Output> {
    None
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
  pub fn get_text(&self) -> String {
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
    let h = self.get_text().lines().count() as f32 * self.get_typeset().line_spacing * self.get_typeset().size as f32;
    vec2!(0, self.style.get_padded_size(vec2!(0, h as u32)).y)
  }

  type Event = H::Output;
  fn draw<L: Layout>(&mut self, screen: &mut Screen<S>, layout: &mut L, focus: &mut SelectId) -> Option<H::Output> {
    // style is resized along with the textbox
    let scissor = layout.apply(self);

    // draw and select
    let e = self.style.draw(screen, layout, focus);
    if let Some(e) = e.as_ref() {
      println!("{:?}", e);
    }
    screen.push_draw(self.text.get_mesh(), scissor);

    H::handle(self, e, screen)
  }
}

#[derive(Default)]
pub struct HandlerReadonly {}
impl TextBoxEventHandler for HandlerReadonly {
  type Output = event::Event;
  fn handle<S: Style>(tb: &mut TextBox<S, Self>, e: Option<event::Event>, screen: &Screen<S>) -> Option<event::Event> {
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
        match e {
          vk::winit::Event::WindowEvent {
            event: vk::winit::WindowEvent::ReceivedCharacter(c),
            ..
          } => {
            // TODO: multiline flag?
            let mut c = *c;
            if c == '\r' {
              c = '\n';
            }
            tb.text(&format!("{}{}", tb.get_text(), c));
          }
          _ => {}
        }
      }
    }

    if let Some(event::Event::Pressed(_)) = e {
      Some(Event::Clicked)
    } else {
      None
    }
  }
}

pub type TextEdit<S, > = TextBox<S, HandlerEdit>;

