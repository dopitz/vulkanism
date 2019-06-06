use super::bindvertexbuffer::BindVertexBuffersTrait;
use crate::cmd::commands::StreamPush;
use crate::cmd::Stream;
use vk;

/// Bind vertex buffers and issues an indexed draw call
#[derive(Default, Debug)]
pub struct DrawIndexed<T: BindVertexBuffersTrait> {
  pub vertex_buffers: T,
  pub index_count: u32,
  pub instance_count: u32,
  pub first_index: u32,
  pub vertex_offset: i32,
  pub first_instance: u32,

  pub index_buffer: vk::Buffer,
  pub index_buffer_offset: vk::DeviceSize,
  pub index_type: vk::IndexType,
}

impl<T: BindVertexBuffersTrait> DrawIndexed<T> {
  /// Creates a new builder for indexed drawing
  ///
  /// Default initializes:
  ///  - `index_count = 0`
  ///  - `instance_count = 1`
  ///  - `first_index = 0`
  ///  - `vertex_offset = 0`
  ///  - `first_instance = 0`
  ///  - `index_buffer_offeset = 0`
  ///  - `index_type = vk::INDEX_TYPE_UINT16`
  pub fn new(vertex_buffers: T, index_buffer: vk::Buffer) -> Self {
    Self {
      vertex_buffers,
      index_count: 0,
      instance_count: 1,
      first_index: 0,
      vertex_offset: 0,
      first_instance: 0,

      index_buffer,
      index_buffer_offset: 0,
      index_type: vk::INDEX_TYPE_UINT16,
    }
  }

  pub fn first_index(mut self, first: u32) -> Self {
    self.first_index = first;
    self
  }
  pub fn index_count(mut self, count: u32) -> Self {
    self.index_count = count;
    self
  }
  pub fn indices(mut self, first: u32, count: u32) -> Self {
    self.first_index = first;
    self.index_count = count;
    self
  }

  pub fn vertex_offset(mut self, offset: i32) -> Self {
    self.vertex_offset = offset;
    self
  }

  pub fn first_instance(mut self, first: u32) -> Self {
    self.first_instance = first;
    self
  }
  pub fn instance_count(mut self, count: u32) -> Self {
    self.instance_count = count;
    self
  }
  pub fn instances(mut self, first: u32, count: u32) -> Self {
    self.first_instance = first;
    self.instance_count = count;
    self
  }

  pub fn buffer_offset(mut self, offset: vk::DeviceSize) -> Self {
    self.index_buffer_offset = offset;
    self
  }
  pub fn index_type(mut self, ty: vk::IndexType) -> Self {
    self.index_type = ty;
    self
  }
}

impl<T: BindVertexBuffersTrait> StreamPush for DrawIndexed<T> {
  fn enqueue(&self, cs: Stream) -> Stream {
    let cs = cs.push(&self.vertex_buffers);
    vk::CmdBindIndexBuffer(cs.buffer, self.index_buffer, self.index_buffer_offset, self.index_type);
    vk::CmdDrawIndexed(
      cs.buffer,
      self.index_count,
      self.instance_count,
      self.first_index,
      self.vertex_offset,
      self.first_instance,
    );
    cs
  }
}
