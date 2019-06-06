use super::bindvertexbuffer::BindVertexBuffersTrait;
use crate::cmd::commands::StreamPush;
use crate::cmd::Stream;
use vk;

/// Binds vertex buffers and issues an indirect draw call
#[derive(Default, Debug)]
pub struct DrawIndirect<T: BindVertexBuffersTrait> {
  pub vertex_buffers: T,
  pub count: u32,
  pub offset: vk::DeviceSize,
  pub stride: u32,
  pub buffer: vk::Buffer,

  pub index_buffer: Option<vk::Buffer>,
  pub index_offset: vk::DeviceSize,
  pub index_type: vk::IndexType,
}

impl<T: BindVertexBuffersTrait> DrawIndirect<T> {
  /// Creates a new builder for indirect drawing
  ///
  /// Default initializes:
  ///  - `count = 0`
  ///  - `offset = 0`
  ///  - `stride = sizeof(vk::DrawIndirectCommand)`
  ///  - `index_buffer = None`
  pub fn new(vertex_buffers: T, buffer: vk::Buffer) -> Self {
    Self {
      vertex_buffers,
      count: 0,
      offset: 0,
      stride: std::mem::size_of::<vk::DrawIndirectCommand>() as u32,
      buffer,

      index_buffer: None,
      index_offset: 0,
      index_type: vk::INDEX_TYPE_UINT16,
    }
  }

  /// Set the builder for indexed indirect drawing
  ///
  /// Also sets the `stride` to teh required `sizeof(vk::DrawIndexedIndirectCommand)`
  pub fn indexed(mut self, index_buffer: vk::Buffer, index_offset: vk::DeviceSize, index_type: vk::IndexType) -> Self {
    self.index_buffer = Some(index_buffer);
    self.index_offset = index_offset;
    self.index_type = index_type;
    self.stride = std::mem::size_of::<vk::DrawIndexedIndirectCommand>() as u32;
    self
  }

  pub fn count(mut self, count: u32) -> Self {
    self.count = count;
    self
  }
  pub fn offset(mut self, offset: vk::DeviceSize) -> Self {
    self.offset = offset;
    self
  }
}

impl<T: BindVertexBuffersTrait> StreamPush for DrawIndirect<T> {
  fn enqueue(&self, cs: Stream) -> Stream {
    let cs = cs.push(&self.vertex_buffers);
    if let Some(indices) = self.index_buffer {
      vk::CmdBindIndexBuffer(cs.buffer, indices, self.index_offset, self.index_type);
      vk::CmdDrawIndexedIndirect(cs.buffer, self.buffer, self.offset, self.count, self.stride);
    } else {
      vk::CmdDrawIndirect(cs.buffer, self.buffer, self.offset, self.count, self.stride);
    }
    cs
  }
}
