mod complete;
mod history;
mod show;
mod window;

use crate::component::textbox::Event as TextboxEvent;
use crate::select::SelectId;
use crate::shell::context::ContextShell;
use crate::style::Style;
use crate::window::Component;
use crate::window::Layout;
use crate::window::Screen;
use crate::ImGui;
use vk::winit;

#[derive(Clone)]
pub struct Terminal<S: Style> {
  pub window: window::TerminalWnd<S>,
  pub show: show::Show<S>,
  pub complete: complete::Complete<S>,
  pub history: history::History<S>,
}

unsafe impl<S: Style> Send for Terminal<S> {}

impl<S: Style> Terminal<S> {
  pub fn new(gui: &ImGui<S>) -> Self {
    let window = window::TerminalWnd::new(gui);
    let show = show::Show::new(window.clone());
    let complete = complete::Complete::new(window.clone());
    let history = history::History::new(window.clone());

    Self {
      window,
      show,
      complete,
      history,
    }
  }

  pub fn draw<L: Layout, C: ContextShell>(&mut self, screen: &mut Screen<S>, layout: &mut L, focus: &mut SelectId, e: Option<&winit::event::Event<i32>>, context: &mut C) {
    let e_wnd = match self.show.get() {
      true => self.window.draw(screen, layout, focus, e),
      false => None,
    };
    self.show.handle_event(e);

    self.complete.handle_event(screen, &e_wnd, e, context);
    self.history.handle_event(screen, &e_wnd, e);

    match e_wnd {
      Some(TextboxEvent::Enter(input)) => {
        context.get_shell().exec(&input, context);
      }
      _ => (),
    };
  }

  /// Print text to the terminal
  pub fn print(&self, s: &str) {
    self.window.print(s);
  }
  /// Print text to the terminal and adds a newline character at the end
  pub fn println(&self, s: &str) {
    self.window.println(s);
  }
  /// Wait until an input is entered into the terminal and return its text.
  ///
  /// **Attention** This will block the current thread and wait for the input.
  /// To not result in a deadlock this function must never be called in the rendering thread that also calls
  /// [draw](struct.Terminal.html#method.draw)
  pub fn readln(&self) -> String {
    self.window.readln()
  }
}
