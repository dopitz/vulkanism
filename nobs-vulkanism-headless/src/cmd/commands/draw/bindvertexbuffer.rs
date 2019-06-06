use super::DrawIndexed;
use super::DrawIndirect;
use super::DrawVertices;
use crate::cmd::commands::StreamPush;
use crate::cmd::Stream;
use vk;

pub trait BindVertexBuffersTrait: StreamPush + Default {}

/// Binds vertex buffers to command stream
#[derive(Debug, Clone, Copy)]
pub struct BindVertexBuffers {
  pub count: u32,
  pub buffers: *const vk::Buffer,
  pub offsets: *const vk::DeviceSize,
}

impl Default for BindVertexBuffers {
  fn default() -> Self {
    Self {
      count: 0,
      buffers: std::ptr::null(),
      offsets: std::ptr::null(),
    }
  }
}

impl BindVertexBuffers {
  pub fn new(count: u32, buffers: *const vk::Buffer, offsets: *const vk::DeviceSize) -> Self {
    Self { count, buffers, offsets }
  }

  pub fn vertices(mut self) -> DrawVertices<Self> {
    DrawVertices::new(self)
  }

  pub fn indexed(mut self, indices: vk::Buffer) -> DrawIndexed<Self> {
    DrawIndexed::new(self, indices)
  }

  pub fn indirect(mut self, buffer: vk::Buffer) -> DrawIndirect<Self> {
    DrawIndirect::new(self, buffer)
  }
}

impl StreamPush for BindVertexBuffers {
  fn enqueue(&self, cs: Stream) -> Stream {
    if self.count > 0 {
      vk::CmdBindVertexBuffers(cs.buffer, 0, self.count, self.buffers, self.offsets);
    }

    cs
  }
}

impl BindVertexBuffersTrait for BindVertexBuffers {}

/// Binds vertex buffers to command stream
#[derive(Debug, Default)]
pub struct BindVertexBuffersManaged {
  pub buffers: Vec<vk::Buffer>,
  pub offsets: Vec<vk::DeviceSize>,
  pub bind: BindVertexBuffers,
}

impl BindVertexBuffersManaged {
  pub fn push(mut self, buffer: vk::Buffer, offset: vk::DeviceSize) -> Self {
    self.buffers.push(buffer);
    self.offsets.push(offset);
    self.bind = BindVertexBuffers::new(self.buffers.len() as u32, self.buffers.as_ptr(), self.offsets.as_ptr());
    self
  }

  pub fn vertices(mut self) -> DrawVertices<Self> {
    DrawVertices::new(self)
  }

  pub fn indexed(mut self, indices: vk::Buffer) -> DrawIndexed<Self> {
    DrawIndexed::new(self, indices)
  }

  pub fn indirect(mut self, buffer: vk::Buffer) -> DrawIndirect<Self> {
    DrawIndirect::new(self, buffer)
  }
}

impl StreamPush for BindVertexBuffersManaged {
  fn enqueue(&self, cs: Stream) -> Stream {
    cs.push(&self.bind)
  }
}

impl BindVertexBuffersTrait for BindVertexBuffersManaged {}
