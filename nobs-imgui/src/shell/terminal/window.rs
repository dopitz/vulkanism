use crate::component::textbox::Event;
use crate::component::*;
use crate::rect::Rect;
use crate::style::Style;
use crate::window::*;
use crate::ImGui;

use std::sync::Arc;
use std::sync::Condvar;
use std::sync::Mutex;

struct TerminalImpl<S: Style> {
  wnd: Window<FloatLayout, S>,

  output_wnd: Window<ColumnLayout, S>,
  output: TextBox<S>,
  pin_scroll: bool,

  input: TextEdit<S>,
  readl: Option<Arc<(Mutex<Option<String>>, Condvar)>>,

  quickfix_wnd: Window<ColumnLayout, S>,
  quickfix: TextEdit<S>,
}

impl<S: Style> Size for TerminalImpl<S> {
  fn set_rect(&mut self, rect: Rect) {
    self.wnd.set_rect(rect);

    let mut outrect = self.wnd.get_client_rect();
    outrect.size.y = outrect.size.y.saturating_sub(self.input.get_size_hint().y + 10);
    self.output_wnd.set_rect(outrect);

    self.output.set_rect(outrect);

    let mut inrect = outrect;
    inrect.position.y = outrect.position.y + outrect.size.y as i32 + 10;
    inrect.size.y = self.input.get_size_hint().y;
    self.input.set_rect(inrect);
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

  fn enqueue<'a, R: std::fmt::Debug>(&mut self, mut s: Stream<'a, S, R>) -> Stream<'a, S, Self::Event> {
    s.layout(self);

    let s = s.push(&mut self.wnd.begin());

    let s = s.push(&mut self.output_wnd.begin());
    if let Some(crate::window::Event::Scroll) = s.get_result() {
      self.pin_scroll = false;
    }
    let s = s.push(&mut self.output);
    let s = s.push(&mut self.output_wnd.end());

    let s = s.push(&mut self.input);
    let tbevent = match s.get_result() {
      Some(Event::Enter(input)) => {
        let input = input[3..].to_string();

        if let Some(readl) = self.readl.take() {
          let &(ref lock, ref cvar) = &*readl;
          let mut inp = lock.lock().unwrap();
          *inp = Some(input.clone());
          cvar.notify_one();
          self.input.text("~$ ");
        } else {
          self.println(&input);
          self.input.text("~$ ");
        }
        Some(Event::Enter(input))
      }
      Some(Event::Changed) => {
        if self.input.get_text().len() < 3 {
          self.input.text("~$ ");
        }
        Some(Event::Changed)
      }
      _ => None,
    };

    match self.input.get_cursor() {
      Some(cp) if cp.x < 3 => {
        self.input.cursor(Some(vec2!(3, 0)));
      }
      _ => (),
    }

    let s = s.push(&mut self.wnd.end());

    //if self.pin_scroll {
    //  self.output_wnd.scroll(vec2!(0, u32::max_value()));
    //}

    let s = if !self.quickfix.get_text().is_empty() {
      let r = self.wnd.get_rect();

      let p = match self.input.get_cursor() {
        Some(cp) => cp,
        None => vec2!(0),
      };

      // restart the layout before assigning the new size, because we want the textbox to fill the exact size window
      // we use the cursor position and text dimension for the window size
      // TODO: is this actually neccessary: self.quickfix_wnd.restart();
      self.quickfix_wnd.set_rect(Rect::new(
        vec2!(
          self.input.get_rect().position.x + self.input.get_typeset().char_offset(self.input.get_text(), p).x as i32,
          r.position.y + r.size.y as i32
        ),
        self.quickfix.get_typeset().text_rect(self.quickfix.get_text()),
      ));

      // draw quickfix window and textbox
      let s = s.push(&mut self.quickfix_wnd.begin()).push(&mut self.quickfix);
      if let Some(_) = s.get_result() {
        self.println("quickfix click not implemented");
      }
      s.push(&mut self.quickfix_wnd.end()).with_result(tbevent)
    } else {
      s.with_result(tbevent)
    };

    if !self.input.has_focus() && (self.wnd.has_focus() || self.output_wnd.has_focus() || self.output.has_focus()) {
      self.focus(true);
    }

    s
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

  #[allow(dead_code)]
  pub fn input_text(&mut self, s: &str) {
    self.input.text(&format!("~$ {}", s)).cursor(Some(vec2!(s.len() as u32 + 3, 0)));
  }
  #[allow(dead_code)]
  pub fn quickfix_text(&mut self, s: &str) {
    self.quickfix.text(s);
  }
}

#[derive(Clone)]
pub struct TerminalWnd<S: Style> {
  term: Arc<Mutex<TerminalImpl<S>>>,
}

unsafe impl<S: Style> Send for TerminalWnd<S> {}

impl<S: Style> Size for TerminalWnd<S> {
  fn set_rect(&mut self, rect: Rect) {
    self.term.lock().unwrap().set_rect(rect);
  }

  fn get_rect(&self) -> Rect {
    self.term.lock().unwrap().get_rect()
  }

  fn get_size_hint(&self) -> vkm::Vec2u {
    self.term.lock().unwrap().get_size_hint()
  }
}

impl<S: Style> Component<S> for TerminalWnd<S> {
  type Event = Event;
  //fn draw<L: Layout>(
  //  &mut self,
  //  screen: &mut Screen<S>,
  //  layout: &mut L,
  //  focus: &mut SelectId,
  //  e: Option<&winit::event::Event<i32>>,
  //) -> Option<Self::Event> {
  //  self.term.lock().unwrap().draw(screen, layout, focus, e)
  //}

  fn enqueue<'a, R: std::fmt::Debug>(&mut self, s: Stream<'a, S, R>) -> Stream<'a, S, Self::Event> {
    s.push(&mut *self.term.lock().unwrap())
  }
}

impl<S: Style> TerminalWnd<S> {
  pub fn new(gui: &ImGui<S>) -> Self {
    let mut wnd = Window::new(gui, FloatLayout::default());
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
    output.text("hello world...\n");

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

  /// Sets focus of the terminal window
  ///
  /// Sets the focus to the input text edit of the terminal and moves the cursor to the right most position.
  /// Same as clicking anywhere in the terminal window.
  pub fn focus(&self, focus: bool) -> &Self {
    self.term.lock().unwrap().focus(focus);
    self
  }
  /// Checks if the terminal window is focused right now
  pub fn has_focus(&self) -> bool {
    self.term.lock().unwrap().input.has_focus()
  }

  /// Sets the size of the terminal window in pixel coordinates
  ///
  /// Size referes to the terminal windows size with borders and caption (if enabled)
  pub fn size(&self, x: u32, y: u32) -> &Self {
    self.term.lock().unwrap().wnd.size(x, y);
    self
  }
  /// Sets the position of the terminal window in pixel coordinates
  ///
  /// The position refercs to the upper left corner of the terminal window
  pub fn position(&self, x: i32, y: i32) -> &Self {
    self.term.lock().unwrap().wnd.position(x, y);
    self
  }

  /// Print text to the terminal
  pub fn print(&self, s: &str) {
    self.term.lock().unwrap().print(s);
  }
  /// Print text to the terminal and adds a newline character at the end
  pub fn println(&self, s: &str) {
    self.term.lock().unwrap().println(s);
  }
  /// Wait until an input is entered into the terminal and return its text.
  ///
  /// **Attention** This will block the current thread and wait for the input.
  /// To not result in a deadlock this function must never be called in the rendering thread that also calls
  /// [draw](struct.TerminalWnd.html#method.draw)
  pub fn readln(&self) -> String {
    // Create new condition variable
    let readl = { self.term.lock().unwrap().readln() };
    let &(ref lock, ref cvar) = &*readl;
    let mut input = lock.lock().unwrap();
    // Wait for condition variable to be signalled when next input is submitted
    while input.is_none() {
      input = cvar.wait(input).unwrap();
    }
    input.take().unwrap()
  }

  /// Set the text of the input edit
  pub fn input_text(&self, s: &str) {
    self
      .term
      .lock()
      .unwrap()
      .input
      .text(&format!("~$ {}", s))
      .cursor(Some(vec2!(s.len() as u32 + 3, 0)));
  }
  /// Get the text of the input edit
  pub fn get_input(&self) -> String {
    self.term.lock().unwrap().input.get_text()[3..].to_owned()
  }

  /// Set the text of the quickfix window
  pub fn quickfix_text(&self, s: &str) {
    self.term.lock().unwrap().quickfix.text(s);
  }
}
