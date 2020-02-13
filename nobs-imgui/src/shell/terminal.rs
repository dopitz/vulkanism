use super::*;
use crate::components::textbox::Event;
use crate::select::SelectId;
use crate::style::Style;
use crate::window::*;

type Am<T> = std::sync::Arc<std::sync::Mutex<T>>;

fn makeam<T>(t: T) -> Am<T> {
  std::sync::Arc::new(std::sync::Mutex::new(t))
}

#[derive(Clone, Copy)]
enum CompleteIndex {
  Input,
  Lcp,
  Index(usize),
}
struct Completion {
  index: CompleteIndex,
  prefix_len: usize,
  reverse: bool,
}
impl Default for Completion {
  fn default() -> Self {
    Self {
      index: CompleteIndex::Input,
      prefix_len: 0,
      reverse: false,
    }
  }
}
impl Completion {
  fn reset(&mut self) {
    self.prefix_len = 0;
    self.index = CompleteIndex::Input;
  }

  fn input<'a, C: Context>(&mut self, input: &'a str, context: &C) -> &'a str {
    self.prefix_len = input.len();
    self.index = CompleteIndex::Input;

    let cmds = context.get_shell().get_commands();
    let completions = if cmds.iter().filter(|c| c.get_name().starts_with(input)).count() > 1 {
      Some(cmds.iter().filter_map(|c| c.complete(input)).flatten().collect::<Vec<_>>())
    } else {
      cmds.iter().find_map(|c| c.complete(input))
    };

    if let Some(completions) = completions {
      let mut s = completions
        .iter()
        .fold(String::new(), |acc, c| format!("{}{}\n", acc, c.get_preview()));
      s = format!("{}{}", s, "-------------");
      &s
    } else {
      ""
    }
  }

  fn handle_event<C: Context>(&mut self, e: &vk::winit::Event, context: &C) {
    match e {
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
      } => match *k {
        vk::winit::VirtualKeyCode::LShift | vk::winit::VirtualKeyCode::RShift => self.reverse = true,
        vk::winit::VirtualKeyCode::Tab => self.next_completion(self.reverse, context),
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
      } => match *k {
        vk::winit::VirtualKeyCode::LShift | vk::winit::VirtualKeyCode::RShift => self.reverse = false,
        _ => (),
      },
      _ => (),
    }
  }

  fn next_completion<C: Context>(&mut self, reverse: bool, context: &C) {
    //let input = self.window.get_input();
    //let mut prefix = input[..self.input.prefix_len].to_string();
    //let completions = self.get_completions(&prefix, context);
    //match completions.as_ref() {
    //  Some(ref completions) if !completions.is_empty() => {
    //    match self.input.complete_index {
    //      CompleteIndex::Input => {
    //        let s = completions[0].get_completed();
    //        let lcp = completions.iter().skip(1).fold(s.len(), |_, c| {
    //          s.chars().zip(c.get_completed().chars()).take_while(|(a, b)| a == b).count()
    //        });
    //        if self.input.prefix_len == lcp {
    //          self.input.complete_index = match reverse {
    //            false => CompleteIndex::Index(0),
    //            true => CompleteIndex::Index(completions.len() - 1),
    //          };
    //          prefix = s[..lcp].to_string();
    //        } else {
    //          self.input.prefix_len = lcp;
    //          self.input.complete_index = CompleteIndex::Lcp;
    //          prefix = s[..lcp].to_string();
    //        }
    //      }
    //      CompleteIndex::Lcp => {
    //        self.input.complete_index = match reverse {
    //          false => CompleteIndex::Index(0),
    //          true => CompleteIndex::Index(completions.len() - 1),
    //        }
    //      }
    //      CompleteIndex::Index(i) => {
    //        let ci = i as i32
    //          + match reverse {
    //            false => 1,
    //            true => -1,
    //          };
    //        self.input.complete_index = if ci < 0 || ci >= completions.len() as i32 {
    //          CompleteIndex::Lcp
    //        } else {
    //          CompleteIndex::Index(ci as usize)
    //        };
    //      }
    //    }

    //    if let CompleteIndex::Index(ci) = self.input.complete_index {
    //      self.window.input_text(&completions[ci].get_completed());
    //    } else {
    //      self.window.input_text(&prefix);
    //    }
    //  }
    //  _ => (),
    //}
  }
}

#[derive(Clone, Copy)]
enum HistoryIndex {
  Input,
  Index(usize),
}
struct History {
  inputs: Vec<String>,
  index: HistoryIndex,
}
impl Default for History {
  fn default() -> Self {
    Self {
      inputs: Default::default(),
      index: HistoryIndex::Input,
    }
  }
}
impl History {
  fn push(&mut self, s: &str) {
    self.inputs.push(s.to_string());
  }

  fn next<'a>(&'a mut self, reverse: bool) -> &'a str {
    if self.inputs.is_empty() {
      return "";
    }
    match self.index {
      HistoryIndex::Input => {
        if reverse {
          self.index = HistoryIndex::Index(self.inputs.len() - 1);
        } else {
          self.index = HistoryIndex::Index(0);
        }
      }
      HistoryIndex::Index(i) => {
        let i = if reverse { i as isize - 1 } else { i as isize + 1 };
        if 0 > i || i as usize >= self.inputs.len() {
          self.index = HistoryIndex::Input;
        } else {
          self.index = HistoryIndex::Index(i as usize);
        }
      }
    }

    if let HistoryIndex::Index(i) = self.index {
      &self.inputs[i]
    } else {
      ""
    }
  }
}

#[derive(Clone)]
pub struct TerminalInput {
  //complete_index: CompleteIndex,
  //prefix_len: usize,
  history: Vec<String>,
  history_index: HistoryIndex,
  //shift: bool,
}

impl Default for TerminalInput {
  fn default() -> Self {
    Self {
      history: Default::default(),
      //prefix_len: 0,
      //complete_index: CompleteIndex::Input,
      history_index: HistoryIndex::Input,
      //shift: false,
    }
  }
}

#[derive(Clone)]
pub struct Terminal<S: Style> {
  pub window: TerminalWnd<S>,
  pub input: TerminalInput,
  show_term: Am<bool>,
  completion: Am<Completion>,
  history: Am<History>,
}

unsafe impl<S: Style> Send for Terminal<S> {}

impl<S: Style> Terminal<S> {
  pub fn new(window: TerminalWnd<S>) -> Self {
    Self {
      window,
      input: TerminalInput::default(),
      show_term: makeam(false),
      completion: makeam(Default::default()),
      history: makeam(Default::default()),
    }
  }

  pub fn draw<L: Layout, C: Context>(&mut self, screen: &mut Screen<S>, layout: &mut L, focus: &mut SelectId, context: &mut C) {
    let e = match *self.show_term.lock().unwrap() {
      true => self.window.draw(screen, layout, focus),
      false => None,
    };
    self.handle_input(e, screen, context);
  }

  pub fn show_term(&mut self, show: bool) {
    *self.show_term.lock().unwrap() = show;
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
        self.completion.lock().unwrap().reset();
        //self.input.prefix_len = 0;
        //self.input.complete_index = CompleteIndex::Input;
        self.window.quickfix_text("");
        Some(input.clone())
      }
      Some(Event::Changed) => {
        self.completion.lock().unwrap().input(&self.window.get_input(), context);
        //let input = self.window.get_input();

        //self.input.prefix_len = input.len();
        //self.input.complete_index = CompleteIndex::Input;

        //if let Some(completions) = self.get_completions(&input, context) {
        //  let mut s = completions
        //    .iter()
        //    .fold(String::new(), |acc, c| format!("{}{}\n", acc, c.get_preview()));
        //  s = format!("{}{}", s, "-------------");
        //  self.window.quickfix_text(&s);
        //} else {
        //  self.window.quickfix_text("");
        //}
        None
      }
      _ => None,
    };

    // handles events that are queued up for the screen
    //  - show/hide terminal window
    //  - cycle completions
    //  - cycle comand history
    let show_term = { *self.show_term.lock().unwrap() };
    let completion = self.completion.lock().unwrap();
    for e in screen.get_events() {
      if show_term {
        completion.handle_event(e, context);
      }

      match e {
        // shows the input/terminal vim-style, when colon is received
        vk::winit::Event::WindowEvent {
          event: vk::winit::WindowEvent::ReceivedCharacter(':'),
          ..
        } if !show_term => self.show_term(true),
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
        } if show_term => match *k {
          vk::winit::VirtualKeyCode::Escape => {
            if self.window.get_input().is_empty() {
              self.show_term(false);
            } else {
              self.window.input_text("");
              self.window.quickfix_text("");
            }
          }

          vk::winit::VirtualKeyCode::Up => self.window.input_text(&self.history.lock().unwrap().next(true)),
          vk::winit::VirtualKeyCode::Down => self.window.input_text(&self.history.lock().unwrap().next(false)),
          _ => (),
        },
        _ => (),
      }
    }

    // execute the command
    if let Some(s) = e {
      context.get_shell().exec(&s, context);
      self.input.history.push(s);
    }
  }
}
