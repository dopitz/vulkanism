use super::*;
use crate::components::textbox::Event;
use crate::select::SelectId;
use crate::style::Style;
use crate::window::*;

#[derive(Clone, Copy)]
enum CompleteIndex {
  Input,
  Lcp,
  Index(usize),
}

#[derive(Clone, Copy)]
enum HistoryIndex {
  Input,
  Index(usize),
}

#[derive(Clone)]
pub struct TerminalInput {
  show_term: bool,

  complete_index: CompleteIndex,
  prefix_len: usize,

  history: Vec<String>,
  history_index: HistoryIndex,
  shift: bool,
}

impl Default for TerminalInput {
  fn default() -> Self {
    Self {
      history: Default::default(),
      show_term: false,
      prefix_len: 0,
      complete_index: CompleteIndex::Input,
      history_index: HistoryIndex::Input,
      shift: false,
    }
  }
}

pub struct Terminal<S: Style> {
  pub window: TerminalWnd<S>,
  pub input: TerminalInput,
}

impl<S: Style> Clone for Terminal<S> {
  fn clone(&self) -> Self {
    Self {
      window: self.window.clone(),
      input: TerminalInput::default(),
    }
  }
}

unsafe impl<S: Style> Send for Terminal<S> {}

impl<S: Style> Terminal<S> {
  pub fn new(window: TerminalWnd<S>) -> Self {
    Self {
      window,
      input: TerminalInput::default(),
    }
  }

  pub fn draw<L: Layout, C: Context>(&mut self, screen: &mut Screen<S>, layout: &mut L, focus: &mut SelectId, context: &mut C) {
    let e = match self.input.show_term {
      true => self.window.draw(screen, layout, focus),
      false => None,
    };
    self.handle_input(e, screen, context);
  }

  pub fn show_term(&mut self, show: bool) {
    self.input.show_term = show;
    self.window.focus(show);
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

  fn handle_input<C: Context>(&mut self, e: Option<crate::components::textbox::Event>, screen: &Screen<S>, context: &mut C) {
    // handles the textbox event from the input box
    let e = match e {
      Some(Event::Enter(input)) => {
        self.input.prefix_len = 0;
        self.input.complete_index = CompleteIndex::Input;
        self.window.quickfix_text("");
        Some(input.clone())
      }
      Some(Event::Changed) => {
        let input = self.window.get_input();
        self.input.prefix_len = input.len();
        self.input.complete_index = CompleteIndex::Input;

        if let Some(completions) = self.get_completions(&input, context) {
          let mut s = completions
            .iter()
            .fold(String::new(), |acc, c| format!("{}{}\n", acc, c.get_preview()));
          s = format!("{}{}", s, "-------------");
          self.window.quickfix_text(&s);
        } else {
          self.window.quickfix_text("");
        }
        None
      }
      _ => None,
    };

    // handles events that are queued up for the screen
    //  - show/hide terminal window
    //  - cycle completions
    //  - cycle comand history
    for e in screen.get_events() {
      match e {
        // shows the input/terminal vim-style, when colon is received
        vk::winit::Event::WindowEvent {
          event: vk::winit::WindowEvent::ReceivedCharacter(':'),
          ..
        } if !self.input.show_term => {
          self.input.show_term = true;
          self.window.focus(true);
        }
        // loose focus for input/hide terminal when esc is pressed
        // cycle through completions with tab
        // cycle through history with up/down arrow
        vk::winit::Event::WindowEvent {
          event:
            vk::winit::WindowEvent::KeyboardInput {
              input:
                vk::winit::KeyboardInput {
                  state: vk::winit::ElementState::Pressed,
                  virtual_keycode: Some(k),
                  ..
                },
              ..
            },
          ..
        } if self.input.show_term => match *k {
          vk::winit::VirtualKeyCode::Escape => {
            if self.window.get_input().is_empty() {
              self.input.show_term = false;
              self.window.focus(false);
            } else {
              self.window.input_text("");
              self.window.quickfix_text("");
            }
          }
          vk::winit::VirtualKeyCode::LShift | vk::winit::VirtualKeyCode::RShift => self.input.shift = true,
          vk::winit::VirtualKeyCode::Tab => self.next_completion(self.input.shift, context),
          vk::winit::VirtualKeyCode::Up => self.next_history(true),
          vk::winit::VirtualKeyCode::Down => self.next_history(false),
          _ => (),
        },
        // register shift pressed/released
        vk::winit::Event::WindowEvent {
          event:
            vk::winit::WindowEvent::KeyboardInput {
              input:
                vk::winit::KeyboardInput {
                  state: vk::winit::ElementState::Released,
                  virtual_keycode: Some(k),
                  ..
                },
              ..
            },
          ..
        } if self.input.show_term => match *k {
          vk::winit::VirtualKeyCode::LShift | vk::winit::VirtualKeyCode::RShift => self.input.shift = false,
          _ => (),
        },
        _ => (),
      }
    }

    // execute the command
    if let Some(s) = e {
      if let Some(exe) = context.get_shell().parse(&s) {
        exe.run(context);
      }
      self.input.history.push(s);
    }
  }

  fn get_completions<C: Context>(&self, input: &str, context: &C) -> Option<Vec<arg::Completion>> {
    let cmds = context.get_shell().get_commands();
    if cmds.iter().filter(|c| c.get_name().starts_with(&input)).count() > 1 {
      Some(cmds.iter().filter_map(|c| c.complete(&input)).flatten().collect::<Vec<_>>())
    } else {
      cmds.iter().find_map(|c| c.complete(&input))
    }
  }
  fn next_completion<C: Context>(&mut self, reverse: bool, context: &C) {
    let input = self.window.get_input();
    let mut prefix = input[..self.input.prefix_len].to_string();
    let completions = self.get_completions(&prefix, context);
    match completions.as_ref() {
      Some(ref completions) if !completions.is_empty() => {
        match self.input.complete_index {
          CompleteIndex::Input => {
            let s = completions[0].get_completed();
            let lcp = completions.iter().skip(1).fold(s.len(), |_, c| {
              s.chars().zip(c.get_completed().chars()).take_while(|(a, b)| a == b).count()
            });
            if self.input.prefix_len == lcp {
              self.input.complete_index = match reverse {
                false => CompleteIndex::Index(0),
                true => CompleteIndex::Index(completions.len() - 1),
              };
              prefix = s[..lcp].to_string();
            } else {
              self.input.prefix_len = lcp;
              self.input.complete_index = CompleteIndex::Lcp;
              prefix = s[..lcp].to_string();
            }
          }
          CompleteIndex::Lcp => {
            self.input.complete_index = match reverse {
              false => CompleteIndex::Index(0),
              true => CompleteIndex::Index(completions.len() - 1),
            }
          }
          CompleteIndex::Index(i) => {
            let ci = i as i32
              + match reverse {
                false => 1,
                true => -1,
              };
            self.input.complete_index = if ci < 0 || ci >= completions.len() as i32 {
              CompleteIndex::Lcp
            } else {
              CompleteIndex::Index(ci as usize)
            };
          }
        }

        if let CompleteIndex::Index(ci) = self.input.complete_index {
          self.window.input_text(&completions[ci].get_completed());
        } else {
          self.window.input_text(&prefix);
        }
      }
      _ => (),
    }
  }

  fn next_history(&mut self, reverse: bool) {
    if self.input.history.is_empty() {
      return;
    }
    match self.input.history_index {
      HistoryIndex::Input => {
        if reverse {
          self.input.history_index = HistoryIndex::Index(self.input.history.len() - 1);
        } else {
          self.input.history_index = HistoryIndex::Index(0);
        }
      }
      HistoryIndex::Index(i) => {
        let i = if reverse { i as isize - 1 } else { i as isize + 1 };
        if 0 > i || i as usize >= self.input.history.len() {
          self.input.history_index = HistoryIndex::Input;
        } else {
          self.input.history_index = HistoryIndex::Index(i as usize);
        }
      }
    }

    if let HistoryIndex::Index(i) = self.input.history_index {
      self.window.input_text(&self.input.history[i]);
    } else {
      self.window.input_text("");
    }
  }
}
