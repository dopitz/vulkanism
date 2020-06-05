use crate::shell::terminal::window::TerminalWnd;
use crate::style::Style;
use std::sync::Arc;
use std::sync::Mutex;
use vk::winit;

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

  pub fn handle_event(&self, e: Option<&winit::event::Event<i32>>) {
    let show = self.get();
    match e {
      // shows the input/terminal vim-style, when colon is received
      Some(vk::winit::event::Event::WindowEvent {
        event: vk::winit::event::WindowEvent::ReceivedCharacter(':'),
        ..
      }) if !show => self.set(true),
      Some(vk::winit::event::Event::WindowEvent {
        event:
          vk::winit::event::WindowEvent::KeyboardInput {
            input:
              vk::winit::event::KeyboardInput {
                state: vk::winit::event::ElementState::Pressed,
                virtual_keycode: Some(vk::winit::event::VirtualKeyCode::Escape),
                ..
              },
            ..
          },
        ..
      }) if show => self.set(false),
      _ => (),
    }
  }
}
