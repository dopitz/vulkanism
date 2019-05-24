use vkm::Vec2i;
use winit::MouseButton;
use winit::VirtualKeyCode;

use std::collections::HashSet;

#[derive(Debug)]
pub struct MouseEvent {
  pub button: MouseButton,
  pub position: Vec2i,
}

#[derive(Debug)]
pub enum Event {
  CharReceived(char),
  KeyUp(VirtualKeyCode),
  KeyDown(VirtualKeyCode),
  MouseMove(Vec2i),
  MouseWheel(Vec2i),
  MouseUp(MouseEvent),
  MouseDown(MouseEvent),
}

impl Event {
  pub fn from_winit_event(e: winit::Event) -> Option<Self> {
    match e {
      winit::Event::WindowEvent {
        event: winit::WindowEvent::ReceivedCharacter(c),
        ..
      } => Some(Event::CharReceived(c)),

      winit::Event::WindowEvent {
        event: winit::WindowEvent::KeyboardInput { input, .. },
        ..
      } => match input.state {
        winit::ElementState::Pressed => input.virtual_keycode.and_then(|k| Some(Event::KeyDown(k))),
        winit::ElementState::Released => input.virtual_keycode.and_then(|k| Some(Event::KeyUp(k))),
      },
      winit::Event::WindowEvent {
        event: winit::WindowEvent::CursorMoved { position, .. },
        ..
      } => Some(Event::MouseMove(vec2!(position.x, position.y).into())),
      winit::Event::WindowEvent {
        event: winit::WindowEvent::MouseWheel {
          delta: winit::MouseScrollDelta::LineDelta(x, y),
          ..
        },
        ..
      } => Some(Event::MouseWheel(vec2!(x, y).into())),
      winit::Event::WindowEvent {
        event: winit::WindowEvent::MouseInput { button, state, .. },
        ..
      } => match state {
        winit::ElementState::Pressed => Some(Event::MouseDown(MouseEvent {
          button,
          position: vec2!(0),
        })),
        winit::ElementState::Released => Some(Event::MouseUp(MouseEvent {
          button,
          position: vec2!(0),
        })),
      },
      _ => None,
    }
  }
}

#[derive(Default)]
pub struct Input {
  keys: HashSet<VirtualKeyCode>,
  buttons: HashSet<MouseButton>,
  cursor: Vec2i,
}

impl Input {
  pub fn parse(&mut self, e: winit::Event) -> Option<Event> {
    match e {
      winit::Event::WindowEvent {
        event: winit::WindowEvent::ReceivedCharacter(c),
        ..
      } => Some(Event::CharReceived(c)),
      winit::Event::WindowEvent {
        event: winit::WindowEvent::KeyboardInput { input, .. },
        ..
      } => match input.state {
        winit::ElementState::Pressed => input.virtual_keycode.and_then(|k| {
          self.keys.insert(k);
          Some(Event::KeyDown(k))
        }),
        winit::ElementState::Released => input.virtual_keycode.and_then(|k| {
          self.keys.remove(&k);
          Some(Event::KeyUp(k))
        }),
      },
      winit::Event::WindowEvent {
        event: winit::WindowEvent::CursorMoved { position, .. },
        ..
      } => {
        self.cursor = vec2!(position.x, position.y).into();
        Some(Event::MouseMove(self.cursor))
      }
      winit::Event::WindowEvent {
        event: winit::WindowEvent::MouseWheel {
          delta: winit::MouseScrollDelta::LineDelta(x, y),
          ..
        },
        ..
      } => Some(Event::MouseWheel(vec2!(x, y).into())),
      winit::Event::WindowEvent {
        event: winit::WindowEvent::MouseInput { button, state, .. },
        ..
      } => match state {
        winit::ElementState::Pressed => {
          self.buttons.insert(button);
          Some(Event::MouseDown(MouseEvent {
            button,
            position: self.cursor,
          }))
        }
        winit::ElementState::Released => {
          self.buttons.remove(&button);
          Some(Event::MouseUp(MouseEvent {
            button,
            position: self.cursor,
          }))
        }
      },
      _ => None,
    }
  }
}
