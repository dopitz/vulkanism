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

pub struct Terminal<S: Style, C> {
  pub window: TerminalWnd<S>,
  pub shell: Shell<S, C>,
  pub input: TerminalInput,
}

impl<S: Style, C> Clone for Terminal<S, C> {
  fn clone(&self) -> Self {
    Self {
      window: self.window.clone(),
      shell: self.shell.clone(),
      input: TerminalInput::default(),
    }
  }
}

unsafe impl<S: Style, C> Send for Terminal<S, C> {}

impl<S: Style, C> Terminal<S, C> {
  pub fn new(window: TerminalWnd<S>, shell: Shell<S, C>) -> Self {
    Self {
      window,
      shell,
      input: TerminalInput::default(),
    }
  }

  pub fn draw<L: Layout>(&mut self, screen: &mut Screen<S>, layout: &mut L, focus: &mut SelectId, context: &mut C) {
    let e = match self.input.show_term {
      true => self.window.draw(screen, layout, focus),
      false => None,
    };
    self.handle_input(e, screen, context);
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

  pub fn exec(&self, c: &str, context: &mut C) {
    self.shell.exec(c, self.clone(), context);
  }

  fn handle_input(&mut self, e: Option<crate::components::textbox::Event>, screen: &Screen<S>, context: &mut C) {
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

        if let Some(completions) = self.get_completions(&input) {
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

    for e in screen.get_events() {
      match e {
        vk::winit::Event::WindowEvent {
          event: vk::winit::WindowEvent::ReceivedCharacter(':'),
          ..
        } if !self.input.show_term => {
          self.input.show_term = true;
          self.window.focus(true);
        }
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
          vk::winit::VirtualKeyCode::Tab => self.next_completion(self.input.shift),
          //vk::winit::VirtualKeyCode::Up => self.history(false, None),
          //vk::winit::VirtualKeyCode::Down => self.history(true, None),
          _ => (),
        },
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

    if let Some(s) = e {
      self.exec(&s, context);
    }
  }

  fn get_completions(&self, input: &str) -> Option<Vec<arg::Completion>> {
    let cmds = self.shell.get_commands();
    if cmds.iter().filter(|c| c.get_name().starts_with(&input)).count() > 1 {
      Some(cmds.iter().filter_map(|c| c.complete(&input)).flatten().collect::<Vec<_>>())
    } else {
      cmds.iter().find_map(|c| c.complete(&input))
    }
  }
  fn next_completion(&mut self, reverse: bool) {
    let input = self.window.get_input();
    let mut prefix = input[..self.input.prefix_len].to_string();
    match self.get_completions(&prefix) {
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
}
