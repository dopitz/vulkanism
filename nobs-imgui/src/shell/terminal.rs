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
  TabComplete(bool),
  InputChanged,
  InputSubmit(String),
}

struct TerminalImpl<S: Style> {
  wnd: Window<ColumnLayout, S>,

  output_wnd: Window<ColumnLayout, S>,
  output: TextBox<S>,

  input: TextBox<S, HandlerTerminalEdit>,

  readline: Arc<(Mutex<Option<String>>, Condvar)>,

  quickfix_wnd: Window<ColumnLayout, S>,
  quickfix: TextEdit<S>,
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

    self.wnd.draw(screen, layout, focus);

    self.output_wnd.draw(screen, &mut self.wnd, focus);
    self.output.draw(screen, &mut self.output_wnd, focus);

    Spacer::new(vec2!(10)).draw(screen, &mut self.wnd, focus);

    let e = match self.input.draw(screen, &mut self.wnd, focus) {
      Some(TerminalInputEvent::TextBox(textbox::Event::Enter)) => {
        let input = self.input.get_text()[3..].to_owned();
        let s = format!("{}{}\n", self.output.get_text(), input);
        self.output.text(&s);
        self.output_wnd.scroll(vec2!(0, u32::max_value()));
        self.input.text("~$ ");

        let &(ref s, ref cv) = &*self.readline;
        let mut s = s.lock().unwrap();
        *s = Some(input.clone());
        cv.notify_one();
        Some(Event::InputSubmit(input))
      }
      Some(TerminalInputEvent::TextBox(textbox::Event::Changed)) => {
        if self.input.get_text().len() < 3 {
          self.input.text("~$ ");
        }
        match self.input.get_cursor() {
          Some(cp) if cp.x < 3 => {
            self.input.cursor(Some(vec2!(3, 0)));
          }
          _ => (),
        }
        Some(Event::InputChanged)
      }
      Some(TerminalInputEvent::TabComplete(s)) => Some(Event::TabComplete(s)),
      _ => None,
    };

    if !self.quickfix.get_text().is_empty() {
      let r = self.wnd.get_rect();

      let p = match self.input.get_cursor() {
        Some(cp) => cp,
        None => vec2!(0),
      };

      let x = self.input.get_rect().position.x + self.input.get_typeset().char_offset(self.input.get_text(), p).x as i32;

      self
        .quickfix_wnd
        .rect(Rect::new(vec2!(x, r.position.y + r.size.y as i32), vec2!(200, 200)));
      self.quickfix_wnd.draw(screen, layout, focus);
      if let Some(_) = self.quickfix.draw(screen, &mut self.quickfix_wnd, focus) {
        self.println("quickfix click not implemented");
      }
    }

    if !self.input.has_focus() && (self.wnd.has_focus() || self.output_wnd.has_focus() || self.output.has_focus()) {
      self.focus(true);
    }

    e
  }
}

impl<S: Style> TerminalImpl<S> {
  pub fn focus(&mut self, focus: bool) {
    let cp = Some(vec2!(self.input.get_text().len() as u32, 0));
    self.input.focus(focus).cursor(cp);
    self.output_wnd.focus(focus);
    self.wnd.focus(focus);
  }

  pub fn print(&mut self, s: &str) {
    let s = format!("{}{}", self.output.get_text(), s);
    self.output.text(&s);
    self.output_wnd.scroll(vec2!(0, u32::max_value()));
    println!("AOEUAOEU");
  }
  pub fn println(&mut self, s: &str) {
    let s = format!("{}{}\n", self.output.get_text(), s);
    let s = self.output.get_typeset().wrap_text(&s, self.output.get_rect().size.x);
    self.output.text(&s);
    self.output_wnd.scroll(vec2!(0, u32::max_value()));
    println!("AOEUAOEU");
  }
}

#[derive(Clone)]
pub struct Terminal<S: Style> {
  term: Arc<Mutex<TerminalImpl<S>>>,
}

unsafe impl<S: Style> Send for Terminal<S> {}

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
    output.style("NoStyle").text("\n");

    let mut input = TextBox::new(gui);
    input.text("~$ ");

    let mut quickfix_wnd = Window::new(gui, ColumnLayout::default());
    quickfix_wnd.draw_caption(false);
    quickfix_wnd.style("NoStyle", false, false);
    let mut quickfix = TextBox::new(gui);
    quickfix.style("TextBoxBorderless").text("");

    Self {
      term: Arc::new(Mutex::new(TerminalImpl {
        wnd,
        output_wnd,
        output,
        input,
        readline: Arc::new((Mutex::new(None), Condvar::new())),
        quickfix_wnd,
        quickfix,
      })),
    }
  }

  pub fn position(&self, x: i32, y: i32) -> &Self {
    self.term.lock().unwrap().wnd.position(x, y);
    self
  }
  pub fn size(&self, x: u32, y: u32) -> &Self {
    self.term.lock().unwrap().wnd.size(x, y);
    self
  }

  pub fn focus(&self, focus: bool) -> &Self {
    self.term.lock().unwrap().focus(focus);
    self
  }

  pub fn print(&self, s: &str) {
    self.term.lock().unwrap().print(s);
  }
  pub fn println(&self, s: &str) {
    self.term.lock().unwrap().println(s);
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

  pub fn input_text(&self, s: &str) {
    self
      .term
      .lock()
      .unwrap()
      .input
      .text(&format!("~$ {}", s))
      .cursor(Some(vec2!(s.len() as u32 + 3, 0)));;
  }
  pub fn quickfix_text(&self, s: &str) {
    self.term.lock().unwrap().quickfix.text(s);
  }
}

use crate::style::event;

#[derive(Debug)]
enum TerminalInputEvent {
  TextBox(textbox::Event),
  TabComplete(bool),
}

#[derive(Default)]
struct HandlerTerminalEdit {}
impl textbox::TextBoxEventHandler for HandlerTerminalEdit {
  type Output = TerminalInputEvent;
  fn handle<S: Style>(tb: &mut TextBox<S, Self>, e: Option<event::Event>, screen: &Screen<S>) -> Option<Self::Output> {
    if tb.has_focus() {
      for e in screen.get_events() {
        if let vk::winit::Event::DeviceEvent {
          event:
            vk::winit::DeviceEvent::Key(vk::winit::KeyboardInput {
              state: vk::winit::ElementState::Pressed,
              virtual_keycode: Some(vk::winit::VirtualKeyCode::Tab),
              modifiers: vk::winit::ModifiersState { shift: s, .. },
              ..
            }),
          ..
        } = e
        {
          return Some(TerminalInputEvent::TabComplete(*s));
        }
        if let Some(e) = Self::receive_character(tb, e, false, &['\t', '\u{1b}']) {
          return Some(TerminalInputEvent::TextBox(e));
        }
        Self::move_cursor(tb, e);
        if let Some(mut c) = tb.get_cursor() {
          if c.x < 3 {
            c.x = 3;
            tb.cursor(Some(c));
          }
        }
      }
    }
    Self::set_cursor(tb, e).map(|e| TerminalInputEvent::TextBox(e))
  }
}
