use crate::cmd::commands::StreamPush;
use crate::cmd::CmdBuffer;

/// Binds vertex buffers and issues draw call
#[derive(Debug, Clone, Copy)]
pub struct DrawVertices {
  pub vertex_count: u32,
  pub instance_count: u32,
  pub first_vertex: u32,
  pub first_instance: u32,
}

impl Default for DrawVertices {
  fn default() -> Self {
    Self {
      vertex_count: 0,
      instance_count: 1,
      first_vertex: 0,
      first_instance: 0,
    }
  }
}

impl DrawVertices {
  pub fn with_vertices(vertex_count: u32) -> Self {
    Self {
      vertex_count,
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

impl StreamPush for DrawVertices {
  fn enqueue(&self, cs: CmdBuffer) -> CmdBuffer {
    if self.vertex_count > 0 && self.instance_count > 0 {
      vk::CmdDraw(
        cs.buffer,
        self.vertex_count,
        self.instance_count,
        self.first_vertex,
        self.first_instance,
      );
    }
    cs
  }
}

