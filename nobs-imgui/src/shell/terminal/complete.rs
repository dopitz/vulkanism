use crate::components::textbox::Event as TextboxEvent;
use crate::shell::command::args;
use crate::shell::terminal::window::TerminalWnd;
use crate::shell::Context;
use crate::style::Style;
use crate::window::Screen;
use std::sync::Arc;
use std::sync::Mutex;
use vk::winit::Event;

#[derive(Clone, Copy, Debug)]
enum Index {
  Input,
  Complete(usize),
}
struct State {
  index: Index,
  input: String,
  completions: Vec<args::Completion>,
}

#[derive(Clone)]
pub struct Complete<S: Style> {
  window: TerminalWnd<S>,
  state: Arc<Mutex<State>>,
}

impl<S: Style> Complete<S> {
  pub fn new(window: TerminalWnd<S>) -> Self {
    Self {
      window,
      state: Arc::new(Mutex::new(State {
        index: Index::Input,
        input: String::new(),
        completions: Vec::new(),
      })),
    }
  }

  pub fn reset(&self) {
    let mut state = self.state.lock().unwrap();
    state.index = Index::Input;
    self.window.quickfix_text("");
  }

  pub fn handle_events<C: Context>(&self, screen: &mut Screen<S>, e: &Option<TextboxEvent>, context: &C) {
    // handles the textbox event from the input box
    match e {
      Some(TextboxEvent::Enter(input)) => self.reset(),
      Some(TextboxEvent::Changed) => self.update_completions(context),
      _ => (),
    };

    for e in screen.get_events() {
      match e {
        vk::winit::Event::WindowEvent {
          event:
            vk::winit::WindowEvent::KeyboardInput {
              input:
                vk::winit::KeyboardInput {
                  state: vk::winit::ElementState::Pressed,
                  virtual_keycode: Some(vk::winit::VirtualKeyCode::Tab),
                  modifiers: vk::winit::ModifiersState { shift: reverse, .. },
                  ..
                },
              ..
            },
          ..
        } => self.next(*reverse),
        _ => (),
      }
    }
  }

  fn update_completions<C: Context>(&self, context: &C) {
    let mut state = self.state.lock().unwrap();
    let input = self.window.get_input();

    // List of completions from command names, or parsed command arguments
    let cmds = context.get_shell().get_commands();

    state.input = input.clone();
    state.completions.clear();
    for c in cmds.iter() {
      c.parse(&input, Some(&mut state.completions));
    }

    // make sure the complete index stays in bound, if we restricted completion variants.
    match state.index {
      Index::Complete(i) => {
        if i >= state.completions.len() {
          state.index = Index::Input;
        }
      }
      _ => (),
    }

    self.update_quickfix(state.index, &state.completions);
  }

  fn next(&self, reverse: bool) {
    let input = self.window.get_input();

    let mut state = self.state.lock().unwrap();
    let mut index = state.index;

    if !state.completions.is_empty() {
      match index {
        Index::Input => {
          index = match reverse {
            false => Index::Complete(0),
            true => Index::Complete(state.completions.len() - 1),
          };
        }
        Index::Complete(i) => {
          let d = match reverse {
            false => 1,
            true => -1,
          };
          let ci = i as i32 + d;
          index = if ci < 0 || ci >= state.completions.len() as i32 {
            Index::Input
          } else {
            self.update_quickfix(state.index, &state.completions);
            Index::Complete(ci as usize)
          };
        }
      }

      match index {
        Index::Input => self.window.input_text(&state.input),
        Index::Complete(i) => {
          let mut input = state.input.clone();

          let c = &state.completions[i];
          if c.replace_input.start == c.replace_input.end {
            input.push_str(&state.completions[i].completed);
          } else {
            input.replace_range(state.completions[i].replace_input.clone(), &state.completions[i].completed);
          }
          self.window.input_text(&input);
        }
      }
    }

    state.index = index;
  }

  fn update_quickfix(&self, index: Index, completions: &[args::Completion]) {
    if !completions.is_empty() {
      let mut s = completions.first().unwrap().completed.to_string();
      for c in completions.iter().skip(1) {
        s = format!("{}\n{}", s, c.completed);
      }

      match index {
        Index::Input => self.window.quickfix_text(&format!("{}===============\n{}", completions[0].hint, s)),
        Index::Complete(i) => self.window.quickfix_text(&format!("{}===============\n{}", completions[i].hint, s)),
      }
    } else {
      self.window.quickfix_text("");
    }
  }
}
