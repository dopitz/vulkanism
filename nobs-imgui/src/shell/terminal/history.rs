use crate::components::textbox::Event as TextboxEvent;
use crate::shell::command::args;
use crate::shell::terminal::window::TerminalWnd;
use crate::shell::Context;
use crate::style::Style;
use crate::window::Screen;
use std::sync::Arc;
use std::sync::Mutex;
use vk::winit::Event;

struct State {
  index: usize,
  inputs: Vec<String>,
}

#[derive(Clone)]
pub struct History {
  state: Arc<Mutex<State>>,
}

impl History {
  pub fn new() -> Self {
    Self {
      state: Arc::new(Mutex::new(State {
        index: 0,
        inputs: Vec::new(),
      })),
    }
  }

  pub fn handle_events<C: Context, S: Style>(&self, screen: &mut Screen<S>, e: &Option<TextboxEvent>, context: &C) {
    // handles the textbox event from the input box
    for e in screen.get_events() {
      match e {
        vk::winit::Event::WindowEvent {
          event:
            vk::winit::WindowEvent::KeyboardInput {
              input:
                vk::winit::KeyboardInput {
                  state: vk::winit::ElementState::Pressed,
                  virtual_keycode: Some(vk::winit::VirtualKeyCode::Up),
                  ..
                },
              ..
            },
          ..
        } => self.next(false),
        vk::winit::Event::WindowEvent {
          event:
            vk::winit::WindowEvent::KeyboardInput {
              input:
                vk::winit::KeyboardInput {
                  state: vk::winit::ElementState::Pressed,
                  virtual_keycode: Some(vk::winit::VirtualKeyCode::Down),
                  ..
                },
              ..
            },
          ..
        } => self.next(true),
        _ => (),
      }
    }
  }

  fn next(&self, reverse: bool) {
    //let input = self.window.get_input();

    //let mut state = self.state.lock().unwrap();
    //let mut index = state.index;

    //if !state.completions.is_empty() {
    //  match index {
    //    Index::Input => {
    //      index = match reverse {
    //        false => Index::Complete(0),
    //        true => Index::Complete(state.completions.len() - 1),
    //      };
    //    }
    //    Index::Complete(i) => {
    //      let d = match reverse {
    //        false => 1,
    //        true => -1,
    //      };
    //      let ci = i as i32 + d;
    //      index = if ci < 0 || ci >= state.completions.len() as i32 {
    //        Index::Input
    //      } else {
    //        self.update_quickfix(state.index, &state.completions);
    //        Index::Complete(ci as usize)
    //      };
    //    }
    //  }

    //  match index {
    //    Index::Input => self.window.input_text(&state.input),
    //    Index::Complete(i) => {
    //      let mut input = state.input.clone();

    //      let c = &state.completions[i];
    //      if c.replace_input.start == c.replace_input.end {
    //        input.push_str(&state.completions[i].completed);
    //      } else {
    //        input.replace_range(state.completions[i].replace_input.clone(), &state.completions[i].completed);
    //      }
    //      self.window.input_text(&input);
    //    }
    //  }
    //}

    //state.index = index;
  }
}
