use crate::shell::terminal::window::TerminalWnd;
use crate::style::Style;
use crate::window::Screen;
use std::sync::Arc;
use std::sync::Mutex;

#[derive(Clone)]
pub struct Show<S: Style> {
  window: TerminalWnd<S>,
  show: Arc<Mutex<bool>>,
}

impl<S: Style> Show<S> {
  pub fn new(window: TerminalWnd<S>) -> Self {
    Self {
      window,
      show: Arc::new(Mutex::new(false)),
    }
  }

  pub fn get(&self) -> bool {
    *self.show.lock().unwrap()
  }

  pub fn set(&self, show: bool) {
    *self.show.lock().unwrap() = show;
    self.window.focus(show);
  }

  pub fn toggle(&self) {
    let mut show = self.show.lock().unwrap();
    *show = !*show;
    self.window.focus(*show);
  }

  pub fn handle_events(&self, screen: &mut Screen<S>) {
    let show = self.get();
    for e in screen.get_events() {
      match e {
        // shows the input/terminal vim-style, when colon is received
        vk::winit::Event::WindowEvent {
          event: vk::winit::WindowEvent::ReceivedCharacter(':'),
          ..
        } if !show => self.set(true),
        vk::winit::Event::WindowEvent {
          event:
            vk::winit::WindowEvent::KeyboardInput {
              input:
                vk::winit::KeyboardInput {
                  state: vk::winit::ElementState::Pressed,
                  virtual_keycode: Some(vk::winit::VirtualKeyCode::Escape),
                  ..
                },
              ..
            },
          ..
        } if show => self.set(false),
        _ => (),
      }
    }
  }
}
