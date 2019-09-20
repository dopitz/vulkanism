use crate::components::textbox::Event;
use crate::components::*;
use crate::rect::Rect;
use crate::select::SelectId;
use crate::style::Style;
use crate::window::*;
use crate::ImGui;

use std::sync::Arc;
use std::sync::Condvar;
use std::sync::Mutex;

struct TerminalImpl<S: Style> {
  wnd: Window<ColumnLayout, S>,

  output_wnd: Window<ColumnLayout, S>,
  output: TextBox<S>,
  pin_scroll: bool,

  input: TextEdit<S>,
  readl: Option<Arc<(Mutex<Option<String>>, Condvar)>>,

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

    if let Some(crate::window::Event::Scroll) = self.output_wnd.draw(screen, &mut self.wnd, focus) {
      self.pin_scroll = false;
    }
    self.output.draw(screen, &mut self.output_wnd, focus);

    Spacer::new(vec2!(10)).draw(screen, &mut self.wnd, focus);

    let e = match self.input.draw(screen, &mut self.wnd, focus) {
      Some(Event::Enter(input)) => {
        let input = input[3..].to_string();

        if let Some(readl) = self.readl.take() {
          let &(ref lock, ref cvar) = &*readl;
          let mut inp = lock.lock().unwrap();
          *inp = Some(input);
          cvar.notify_one();
          self.input.text("~$ ");
          None
        } else {
          self.println(&input);
          self.input.text("~$ ");
          Some(Event::Enter(input))
        }
      }
      Some(Event::Changed) => {
        if self.input.get_text().len() < 3 {
          self.input.text("~$ ");
        }
        match self.input.get_cursor() {
          Some(cp) if cp.x < 3 => {
            self.input.cursor(Some(vec2!(3, 0)));
          }
          _ => (),
        }
        Some(Event::Changed)
      }
      _ => None,
    };

    if self.pin_scroll {
      self.output_wnd.scroll(vec2!(0, u32::max_value()));
    }

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
    self.pin_scroll = true;
  }
  pub fn println(&mut self, s: &str) {
    let s = format!("{}{}\n", self.output.get_text(), s);
    let s = self.output.get_typeset().wrap_text(&s, self.output.get_rect().size.x);
    self.output.text(&s);
    self.pin_scroll = true;
  }

  pub fn readln(&mut self) -> Arc<(Mutex<Option<String>>, Condvar)> {
    let readl = Arc::new((Mutex::new(None), Condvar::new()));
    self.readl = Some(readl.clone());
    readl
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
        pin_scroll: true,
        input,
        readl: None,
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
  pub fn get_input(&self) -> String {
    self.term.lock().unwrap().input.get_text()[3..].to_owned()
  }
  pub fn readln(&self) -> String {
    // Wait for the thread to start up.
    let readl = { self.term.lock().unwrap().readln() };
    let &(ref lock, ref cvar) = &*readl;
    let mut input = lock.lock().unwrap();
    while input.is_none() {
      input = cvar.wait(input).unwrap();
    }
    input.take().unwrap()
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
