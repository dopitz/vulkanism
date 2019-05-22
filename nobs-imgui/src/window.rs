use vk;
use vk::builder::Buildable;

use crate::rect::Rect;

pub trait Component: vk::cmd::commands::StreamPush {
  fn add_compontent(&mut self, wnd: &mut Window);
}

pub struct Window<'a> {
  pub device: vk::Device,
  pub ub_viewport: vk::Buffer,

  pub rect: Rect,

  scissor: vk::cmd::commands::Scissor,

  components: Vec<&'a mut Component>,
}

impl<'a> Window<'a> {
  pub fn new(device: vk::Device, ub_viewport: vk::Buffer) -> Window<'a> {
    Self {
      device,
      ub_viewport,
      rect: Default::default(),
      scissor: vk::cmd::commands::Scissor::with_size(0, 0),
      components: Default::default(),
    }
  }

  pub fn size(mut self, w: u32, h: u32) -> Self {
    self.scissor.rect.extent = vk::Extent2D::build().set(w, h).extent;
    self.rect.size = vkm::Vec2::new(w, h);
    self
  }

  pub fn position(mut self, x: i32, y: i32) -> Self {
    self.scissor.rect.offset = vk::Offset2D::build().set(x, y).offset;
    self.rect.position = vkm::Vec2::new(x, y);
    self
  }

  pub fn get_next_bounds(&mut self) -> Rect {
    self.rect
  }

  pub fn push(mut self, c: &'a mut Component) -> Self {
    c.add_compontent(&mut self);
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

    for c in self.components.iter() {
      cs = c.enqueue(cs);
    }
    cs
  }
}
