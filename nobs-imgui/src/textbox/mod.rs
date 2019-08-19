use crate::font::*;
use crate::rect::Rect;
use crate::select::rects::RectId;
use crate::select::SelectId;
use crate::style::event;
use crate::style::Style;
use crate::style::StyleComponent;
use crate::text::Text;
use crate::window::Component;
use crate::window::Layout;
use crate::window::Screen;
use crate::ImGui;
use vk::pass::MeshId;

#[derive(Debug)]
pub enum Event {
  Clicked,
  MouseOver,
  Changed,
}

pub struct TextBox<S: Style> {
  text: Text<S>,
  style: S::Component,
}

impl<S: Style> TextBox<S> {
  pub fn new(gui: &ImGui<S>) -> Self {
    let text = Text::new(gui);
    let style = S::Component::new(gui, "TextBox".to_owned(), false, false);
    Self { text, style }
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

  //  pub fn style(&mut self, style: &str) -> &mut Self {
  //    self.style.set_style(style);
  //    self
  //  }

  pub fn typeset(&mut self, ts: TypeSet) -> &mut Self {
    self.text.typeset(ts);
    self
  }
  pub fn get_typeset(&self) -> TypeSet {
    self.text.get_typeset()
  }
}

impl<S: Style> Component<S> for TextBox<S> {
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

  type Event = Event;
  fn draw<L: Layout>(&mut self, screen: &mut Screen<S>, layout: &mut L, focus: &mut SelectId) -> Option<Event> {
    // style is resized along with the textbox
    let scissor = layout.apply(self);

    // draw and select
    let e = self.style.draw(screen, layout, focus);
    screen.push_draw(self.text.get_mesh(), scissor);

    if self.style.has_focus() {
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
            self.text(&format!("{}{}", self.get_text(), c));
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

pub struct TextEdit<S: Style> {
  tb: TextBox<S>,
}

impl<S: Style> TextEdit<S> {
  pub fn new(gui: &ImGui<S>) -> Self {
    let mut tb = TextBox::new(gui);
    //tb.style("TextEdit");
    Self { tb }
  }

  pub fn get_gui(&self) -> ImGui<S> {
    self.tb.get_gui()
  }

  pub fn text(&mut self, text: &str) -> &mut Self {
    self.tb.text(text);
    self
  }
  pub fn get_text(&self) -> String {
    self.tb.get_text()
  }

  //  pub fn style(&mut self, style: &str) -> &mut Self {
  //    self.tb.set_style(style);
  //    self
  //  }

  pub fn typeset(&mut self, ts: TypeSet) -> &mut Self {
    self.tb.typeset(ts);
    self
  }
  pub fn get_typeset(&self) -> TypeSet {
    self.tb.get_typeset()
  }
}

impl<S: Style> Component<S> for TextEdit<S> {
  fn rect(&mut self, rect: Rect) -> &mut Self {
    self.tb.rect(rect);
    self
  }
  fn get_rect(&self) -> Rect {
    self.tb.get_rect()
  }

  fn get_size_hint(&self) -> vkm::Vec2u {
    self.tb.get_size_hint()
  }

  type Event = Event;
  fn draw<L: Layout>(&mut self, screen: &mut Screen<S>, layout: &mut L, focus: &mut SelectId) -> Option<Event> {
    self.tb.draw(screen, layout, focus);
    None

    //// style is resized along with the textbox
    //let scissor = layout.apply(self);

    //// draw and select
    //let e = self.style.draw(screen, layout, focus);
    //screen.push_draw(self.text.get_mesh(), scissor);


    //if self.style.has_focus() {
    //  for e in screen.get_events() {
    //    match e {
    //      vk::winit::Event::WindowEvent {
    //        event: vk::winit::WindowEvent::ReceivedCharacter(c),
    //        ..
    //      } => {
    //        // TODO: multiline flag?
    //        let mut c = *c;
    //        if c == '\r' {
    //          c = '\n';
    //        }
    //        self.text(&format!("{}{}", self.get_text(), c));
    //      }
    //      _ => {}
    //    }
    //  }
    //}
  }
}
