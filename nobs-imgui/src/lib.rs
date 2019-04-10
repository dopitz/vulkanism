extern crate cgmath as cgm;
extern crate nobs_vulkanism_headless as vk;

pub mod text;

pub struct ImGui {
  screen: Window,
  current: Option<Window>,
}

struct Window {
  pub ub_viewport: vk::Buffer,
  pub viewport: vk::cmd::commands::Viewport,
  pub scissors: vk::cmd::commands::Scissor,
}

impl ImGui {
  pub fn new() -> Self {
    ImGui { screen: Window {}, None }
  }

  pub fn resize() {

  }

  fn get_current() -> Window {
    screen
  }
}
