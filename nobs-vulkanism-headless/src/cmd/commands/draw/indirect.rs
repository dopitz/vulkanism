use super::bindvertexbuffer::BindVertexBuffersTrait;
use crate::cmd::commands::StreamPush;
use crate::cmd::Stream;
use vk;

/// Binds vertex buffers and issues an indirect draw call
#[derive(Debug, Clone, Copy)]
pub struct DrawIndirect {
  pub count: u32,
  pub offset: vk::DeviceSize,
  pub stride: u32,
  pub buffer: vk::Buffer,

  pub index_buffer: Option<vk::Buffer>,
  pub index_offset: vk::DeviceSize,
  pub index_type: vk::IndexType,
}

impl Default for DrawIndirect {
  fn default() -> Self {
    Self {
      count: 0,
      offset: 0,
      stride: std::mem::size_of::<vk::DrawIndirectCommand>() as u32,
      buffer: vk::NULL_HANDLE,

      index_buffer: None,
      index_offset: 0,
      index_type: vk::INDEX_TYPE_UINT16,
    }
  }
}

impl DrawIndirect {
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

impl StreamPush for DrawIndirect {
  fn enqueue(&self, cs: Stream) -> Stream {
    if let Some(indices) = self.index_buffer {
      vk::CmdBindIndexBuffer(cs.buffer, indices, self.index_offset, self.index_type);
      vk::CmdDrawIndexedIndirect(cs.buffer, self.buffer, self.offset, self.count, self.stride);
    } else {
      vk::CmdDrawIndirect(cs.buffer, self.buffer, self.offset, self.count, self.stride);
    }
    cs
  }
}
