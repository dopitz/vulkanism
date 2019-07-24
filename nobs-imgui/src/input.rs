use vkm::Vec2i;
use vk::winit::MouseButton;
use vk::winit::VirtualKeyCode;

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
  pub fn from_winit_event(e: vk::winit::Event) -> Option<Self> {
    match e {
      vk::winit::Event::WindowEvent {
        event: vk::winit::WindowEvent::ReceivedCharacter(c),
        ..
      } => Some(Event::CharReceived(c)),

      vk::winit::Event::WindowEvent {
        event: vk::winit::WindowEvent::KeyboardInput { input, .. },
        ..
      } => match input.state {
        vk::winit::ElementState::Pressed => input.virtual_keycode.and_then(|k| Some(Event::KeyDown(k))),
        vk::winit::ElementState::Released => input.virtual_keycode.and_then(|k| Some(Event::KeyUp(k))),
      },
      vk::winit::Event::WindowEvent {
        event: vk::winit::WindowEvent::CursorMoved { position, .. },
        ..
      } => Some(Event::MouseMove(vec2!(position.x, position.y).into())),
      vk::winit::Event::WindowEvent {
        event: vk::winit::WindowEvent::MouseWheel {
          delta: vk::winit::MouseScrollDelta::LineDelta(x, y),
          ..
        },
        ..
      } => Some(Event::MouseWheel(vec2!(x, y).into())),
      vk::winit::Event::WindowEvent {
        event: vk::winit::WindowEvent::MouseInput { button, state, .. },
        ..
      } => match state {
        vk::winit::ElementState::Pressed => Some(Event::MouseDown(MouseEvent {
          button,
          position: vec2!(0),
        })),
        vk::winit::ElementState::Released => Some(Event::MouseUp(MouseEvent {
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
  pub fn parse(&mut self, e: vk::winit::Event) -> Option<Event> {
    match e {
      vk::winit::Event::WindowEvent {
        event: vk::winit::WindowEvent::ReceivedCharacter(c),
        ..
      } => Some(Event::CharReceived(c)),
      vk::winit::Event::WindowEvent {
        event: vk::winit::WindowEvent::KeyboardInput { input, .. },
        ..
      } => match input.state {
        vk::winit::ElementState::Pressed => input.virtual_keycode.and_then(|k| {
          self.keys.insert(k);
          Some(Event::KeyDown(k))
        }),
        vk::winit::ElementState::Released => input.virtual_keycode.and_then(|k| {
          self.keys.remove(&k);
          Some(Event::KeyUp(k))
        }),
      },
      vk::winit::Event::WindowEvent {
        event: vk::winit::WindowEvent::CursorMoved { position, .. },
        ..
      } => {
        self.cursor = vec2!(position.x, position.y).into();
        Some(Event::MouseMove(self.cursor))
      }
      vk::winit::Event::WindowEvent {
        event: vk::winit::WindowEvent::MouseWheel {
          delta: vk::winit::MouseScrollDelta::LineDelta(x, y),
          ..
        },
        ..
      } => Some(Event::MouseWheel(vec2!(x, y).into())),
      vk::winit::Event::WindowEvent {
        event: vk::winit::WindowEvent::MouseInput { button, state, .. },
        ..
      } => match state {
        vk::winit::ElementState::Pressed => {
          self.buttons.insert(button);
          Some(Event::MouseDown(MouseEvent {
            button,
            position: self.cursor,
          }))
        }
        vk::winit::ElementState::Released => {
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
