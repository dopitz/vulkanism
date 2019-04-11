extern crate cgmath as cgm;
extern crate nobs_vulkanism_headless as vk;

pub mod text;

pub struct ImGui {
  pub device: vk::Device,
  pub queue_copy: vk::Queue,
  pub cmds: vk::cmd::Pool,
  pub pass: vk::RenderPass,
  pub subpass: u32,
  pub alloc: vk::mem::Allocator,
  //screen: Window,
  //current: Option<Window>,
}

//struct Window {
//  pub ub_viewport: vk::Buffer,
//  pub viewport: vk::cmd::commands::Viewport,
//  pub scissors: vk::cmd::commands::Scissor,
//}

impl ImGui {
  pub fn new(
    device: vk::Device,
    queue_copy: vk::Queue,
    cmds: vk::cmd::Pool,
    pass: vk::RenderPass,
    subpass: u32,
    alloc: vk::mem::Allocator,
  ) -> Self {
    ImGui {
      device,
      queue_copy,
      cmds,
      pass,
      subpass,
      alloc,
    }
  }

  //pub fn resize() {

  //}

  //fn get_current() -> Window {
  //  screen
  //}
}
