use std::sync::Arc;

use vk;
use vk::builder::Buildable;

use crate::ImGui;

pub trait Component: vk::cmd::commands::StreamPush {
  fn add_compontent(&mut self, wnd: &Window);
}

pub struct Window<'a> {
  pub device: vk::Device,
  pub ub_viewport: vk::Buffer,

  scissor: vk::cmd::commands::Scissor,

  components: Vec<&'a mut Component>,
}

impl<'a> Window<'a> {
  pub fn new(device: vk::Device, ub_viewport: vk::Buffer) -> Window<'a>  {
    Window {
      device,
      ub_viewport,
      scissor: vk::cmd::commands::Scissor::with_size(0,0),
      components: Default::default(),
    }
  }

  pub fn size(mut self, w: u32, h: u32) -> Self {
    self.scissor.rect.extent = vk::Extent2D::build().set(w, h).extent;
    self
  }

  pub fn position(mut self, x: i32, y: i32) -> Self {
    self.scissor.rect.offset = vk::Offset2D::build().set(x, y).offset;
    self
  }


  pub fn push(mut self, c: &'a mut Component) -> Self {
    c.add_compontent(&self);
    self.components.push(c);
    self
  }

  pub fn clear(&mut self) {
    self.components.clear();
  }
}

impl<'a> vk::cmd::commands::StreamPush for Window<'a> {
  fn enqueue(&self, cs: vk::cmd::Stream) -> vk::cmd::Stream {
    let mut cs = cs.push(&self.scissor);
    println!("{:?}", self.scissor.rect);

    for c in self.components.iter() {
      cs = c.enqueue(cs);
    }
    cs
  }
}
