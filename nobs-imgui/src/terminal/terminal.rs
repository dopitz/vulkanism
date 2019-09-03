use crate::components::*;
use crate::rect::Rect;
use crate::select::SelectId;
use crate::style::Style;
use crate::window::*;
use crate::ImGui;

use std::sync::Arc;
use std::sync::Condvar;
use std::sync::Mutex;

#[derive(Debug, Clone)]
pub enum Event {
  InputChanged,
  InputSubmit(String),
}

struct TerminalImpl<S: Style> {
  wnd: Window<ColumnLayout, S>,

  output_wnd: Window<ColumnLayout, S>,
  output: TextBox<S>,

  input: TextEdit<S>,

  readline: Arc<(Mutex<Option<String>>, Condvar)>,
}

impl<S: Style> Size for TerminalImpl<S> {
  fn rect(&mut self, rect: Rect) -> &mut Self {
    self.wnd.rect(rect);
    let mut r = self.wnd.get_client_rect();
    r.size.y = r.size.y.saturating_sub(self.input.get_size_hint().y + 10);
    self.output_wnd.rect(r);
    self
  }

  fn get_rect(&self) -> Rect {
    self.wnd.get_rect()
  }

  fn get_size_hint(&self) -> vkm::Vec2u {
    self.wnd.get_size_hint()
  }
}

impl<S: Style> Component<S> for TerminalImpl<S> {
  type Event = Event;
  fn draw<L: Layout>(&mut self, screen: &mut Screen<S>, layout: &mut L, focus: &mut SelectId) -> Option<Self::Event> {
    layout.apply(self);

    let mut set_focused = false;
    if let Some(_) = self.wnd.draw(screen, layout, focus) {
      set_focused = true;
    }

    self.output_wnd.draw(screen, &mut self.wnd, focus);
    if let Some(_) = self.output.draw(screen, &mut self.output_wnd, focus) {
      set_focused = true;
    }

    Spacer::new(vec2!(10)).draw(screen, &mut self.wnd, focus);

    let e = match self.input.draw(screen, &mut self.wnd, focus) {
      Some(textbox::Event::Enter) => {
        let input = self.input.get_text()[3..].to_owned();
        let s = format!("{}{}\n", self.output.get_text(), input);
        self.output.text(&s);
        self.input.text("~$ ");

        let &(ref s, ref cv) = &*self.readline;
        let mut s = s.lock().unwrap();
        *s = Some(input.clone());
        cv.notify_one();
        Some(Event::InputSubmit(input))
      }
      Some(textbox::Event::Changed) => Some(Event::InputChanged),
      Some(_) | None => {
        set_focused = true;
        None
      }
    };

    if set_focused {
      let cp = Some(vec2!(self.input.get_text().len() as u32, 0));
      self.input.focus(true).cursor(cp);
      self.output_wnd.focus(true);
    }

    e
  }
}

#[derive(Clone)]
pub struct Terminal<S: Style> {
  term: Arc<Mutex<TerminalImpl<S>>>,
}

impl<S: Style> Size for Terminal<S> {
  fn rect(&mut self, rect: Rect) -> &mut Self {
    self.term.lock().unwrap().rect(rect);
    self
  }

  fn get_rect(&self) -> Rect {
    self.term.lock().unwrap().get_rect()
  }

  fn get_size_hint(&self) -> vkm::Vec2u {
    self.term.lock().unwrap().get_size_hint()
  }
}

impl<S: Style> Component<S> for Terminal<S> {
  type Event = Event;
  fn draw<L: Layout>(&mut self, screen: &mut Screen<S>, layout: &mut L, focus: &mut SelectId) -> Option<Self::Event> {
    self.term.lock().unwrap().draw(screen, layout, focus)
  }
}

impl<S: Style> Terminal<S> {
  pub fn new(gui: &ImGui<S>) -> Self {
    let mut wnd = Window::new(gui, ColumnLayout::default());
    wnd
      .caption("terminal")
      .position(20, 20)
      .size(500, 500)
      .focus(true)
      .draw_caption(false);

    let mut output_wnd = Window::new(gui, ColumnLayout::default());
    output_wnd.draw_caption(false);
    output_wnd.style("NoStyle", false, false);
    let mut output = TextBox::new(gui);
    output.style("NoStyle").text("Welcome!\n");

    let mut input = TextBox::new(gui);
    input.text("~$ ");

    Self {
      term: Arc::new(Mutex::new(TerminalImpl {
        wnd,
        output_wnd,
        output,
        input,
        readline: Arc::new((Mutex::new(None), Condvar::new())),
      })),
    }
  }

  //fn draw(&mut self, screen: &mut Screen<S>, focus: &mut SelectId) -> Option<Event> {
  //  self.term.lock().unwrap().draw(screen, screen.get_layout(), focus)
  //}

  pub fn position(&mut self, x: i32, y: i32) -> &mut Self {
    self.term.lock().unwrap().wnd.position(x, y);
    self
  }
  pub fn size(&mut self, x: u32, y: u32) -> &mut Self {
    self.term.lock().unwrap().wnd.size(x, y);
    self
  }

  pub fn print(&self, s: &str) {
    let mut term = self.term.lock().unwrap();
    let s = format!("{}{}", term.output.get_text(), s);
    term.output.text(&s);
  }
  pub fn println(&self, s: &str) {
    let mut term = self.term.lock().unwrap();
    let s = format!("{}{}\n", term.output.get_text(), s);
    term.output.text(&s);
  }
  pub fn readln(&self) -> String {
    let &(ref s, ref cv) = &*self.term.lock().unwrap().readline;
    let mut s = s.lock().unwrap();
    while s.is_none() {
      s = cv.wait(s).unwrap();
    }
    // TODO cond var thats getting signaled when input returns Enter event
    s.take().unwrap().to_owned()
  }
  pub fn get_input(&self) -> String {
    self.term.lock().unwrap().input.get_text()[3..].to_owned()
  }
}
