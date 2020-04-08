use crate::components::textbox::Event as TextboxEvent;
use crate::shell::terminal::window::TerminalWnd;
use crate::style::Style;
use crate::window::Screen;
use std::sync::Arc;
use std::sync::Mutex;

struct State {
  index: usize,
  inputs: Vec<String>,
}

#[derive(Clone)]
pub struct History<S: Style> {
  state: Arc<Mutex<State>>,
  window: TerminalWnd<S>,
}

impl<S: Style> History<S> {
  pub fn new(window: TerminalWnd<S>) -> Self {
    Self {
      state: Arc::new(Mutex::new(State {
        index: 0,
        inputs: vec![],
      })),
      window,
    }
  }

  pub fn handle_events(&self, screen: &mut Screen<S>, e: &Option<TextboxEvent>) {
    // On enter, registers the current input text to history
    // On text input, resets the index to the end of history
    match e {
      Some(TextboxEvent::Enter(t)) => {
        let mut state = self.state.lock().unwrap();

        if state.inputs.is_empty() || state.inputs.last().unwrap() != t {
          state.inputs.push(t.to_string());
        }

        state.index = state.inputs.len();
      }
      Some(TextboxEvent::Changed) => {
        let mut state = self.state.lock().unwrap();
        state.index = state.inputs.len();
      }
      _ => (),
    };

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
    let mut state = self.state.lock().unwrap();

    match reverse {
      true => {
        state.index += 1;
        if state.index == state.inputs.len() {
          state.index = 0;
        }
      }

      false => match state.index {
        0 => state.index = state.inputs.len() - 1,
        _ => state.index -= 1,
      },
    }

    self.window.input_text(&state.inputs[state.index]);
  }
}
