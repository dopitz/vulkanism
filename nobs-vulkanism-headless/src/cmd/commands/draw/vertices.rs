use super::bindvertexbuffer::BindVertexBuffersTrait;
use crate::cmd::commands::StreamPush;
use crate::cmd::Stream;

/// Binds vertex buffers and issues draw call
#[derive(Default, Debug)]
pub struct DrawVertices<T: BindVertexBuffersTrait> {
  pub vertex_buffers: T,
  pub vertex_count: u32,
  pub instance_count: u32,
  pub first_vertex: u32,
  pub first_instance: u32,
}

impl<T: BindVertexBuffersTrait> DrawVertices<T> {
  /// Creates a new builder for normal drawing
  ///
  /// Default initializes:
  ///  - `index_count = 0`
  ///  - `instance_count = 1`
  ///  - `first_index = 0`
  ///  - `first_instance = 0`
  pub fn new(vertex_buffers: T) -> Self {
    Self {
      vertex_buffers,
      vertex_count: 0,
      instance_count: 1,
      first_vertex: 0,
      first_instance: 0,
    }
  }

  pub fn first_vertex(mut self, first: u32) -> Self {
    self.first_vertex = first;
    self
  }
  pub fn vertex_count(mut self, count: u32) -> Self {
    self.vertex_count = count;
    self
  }
  pub fn vertices(mut self, first: u32, count: u32) -> Self {
    self.first_vertex = first;
    self.vertex_count = count;
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
}

impl<T: BindVertexBuffersTrait> StreamPush for DrawVertices<T> {
  fn enqueue(&self, cs: Stream) -> Stream {
    let cs = cs.push(&self.vertex_buffers);
    vk::CmdDraw(
      cs.buffer,
      self.vertex_count,
      self.instance_count,
      self.first_vertex,
      self.first_instance,
    );
    cs
  }
}
