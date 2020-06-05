use crate::component::textbox::Event as TextboxEvent;
use crate::shell::command::args;
use crate::shell::context::ContextShell;
use crate::shell::terminal::window::TerminalWnd;
use crate::style::Style;
use crate::window::Screen;
use std::sync::Arc;
use std::sync::Mutex;

#[derive(Clone, Copy, Debug)]
enum Index {
  Input,
  Complete(usize),
}
struct State {
  index: Index,
  input: String,
  completions: Option<Vec<args::Completion>>,
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
        completions: Some(Vec::new()),
      })),
    }
  }

  pub fn reset(&self) {
    let mut state = self.state.lock().unwrap();
    state.index = Index::Input;
    self.window.quickfix_text("");
  }

  pub fn handle_events<C: ContextShell>(&self, screen: &mut Screen<S>, e: &Option<TextboxEvent>, context: &C) {
    // handles the textbox event from the input box
    match e {
      Some(TextboxEvent::Enter(_)) => self.reset(),
      Some(TextboxEvent::Changed) => self.update_completions(context),
      _ => (),
    };

    for e in screen.get_events() {
      match e {
        vk::winit::event::Event::WindowEvent {
          event:
            vk::winit::event::WindowEvent::KeyboardInput {
              input:
                vk::winit::event::KeyboardInput {
                  state: vk::winit::event::ElementState::Pressed,
                  virtual_keycode: Some(vk::winit::event::VirtualKeyCode::Tab),
                  //modifiers: vk::winit::event::ModifiersState { shift: reverse, .. },
                  ..
                },
              ..
            },
          ..
        } => {
          //if self.next(*reverse) {
          if self.next(false) {
            self.update_completions(context);
          }
        }
        _ => (),
      }
    }
  }

  fn update_completions<C: ContextShell>(&self, context: &C) {
    let mut state = self.state.lock().unwrap();
    let input = self.window.get_input();

    // List of completions from command names, or parsed command arguments
    let cmds = context.get_shell().get_commands();

    state.input = input.clone();
    let mut completions = state.completions.take().unwrap();
    completions.clear();

    let mut completions = Some(completions);
    for c in cmds.iter() {
      args::Matches::new(&input, c.get_args(), &mut completions);
    }
    let completions = completions.take().unwrap();

    // make sure the complete index stays in bound, if we restricted completion variants.
    match state.index {
      Index::Complete(i) => {
        if i >= completions.len() {
          state.index = Index::Input;
        }
      }
      _ => (),
    }

    self.update_quickfix(state.index, &completions);
    state.completions = Some(completions);
  }

  fn next(&self, reverse: bool) -> bool {
    let mut state = self.state.lock().unwrap();
    let mut index = state.index;
    let completions = state.completions.take().unwrap();

    let mut update_completions = false;
    if !completions.is_empty() {
      match index {
        Index::Input => {
          let mut longest_prefix = completions[0].complete(&state.input.clone());
          for c in completions.iter().skip(1) {
            let completed = c.complete(&state.input.clone());
            let pos = longest_prefix
              .chars()
              .zip(completed.chars())
              .position(|(p, c)| p != c)
              .unwrap_or(usize::min(longest_prefix.len(), completed.len()));
            if pos < longest_prefix.len() {
              longest_prefix = longest_prefix[0..pos].to_string();
            }
          }

          if longest_prefix.len() == self.window.get_input().len() {
            // check if we can complete to common prefix
            index = match reverse {
              false => Index::Complete(0),
              true => Index::Complete(completions.len() - 1),
            };
          } else {
            state.input = longest_prefix;
            update_completions = true;
          }
        }
        Index::Complete(i) => {
          let d = match reverse {
            false => 1,
            true => -1,
          };
          let ci = i as i32 + d;
          index = if ci < 0 || ci >= completions.len() as i32 {
            Index::Input
          } else {
            self.update_quickfix(state.index, &completions);
            Index::Complete(ci as usize)
          };
        }
      }

      match index {
        Index::Input => self.window.input_text(&state.input),
        Index::Complete(i) => self.window.input_text(&completions[i].complete(&state.input.clone())),
      }
    }

    state.index = index;
    state.completions = Some(completions);
    update_completions
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
