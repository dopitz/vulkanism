use super::BindVertexBuffers;
use super::BindVertexBuffersManaged;
use super::bindvertexbuffer::BindVertexBuffersTrait;
use super::DrawIndexed;
use super::DrawIndirect;
use super::DrawVertices;
use crate::cmd::commands::StreamPush;
use crate::cmd::Stream;

#[derive(Debug)]
pub enum DrawKind<T: BindVertexBuffersTrait> {
  Vertices(DrawVertices<T>),
  Indexed(DrawIndexed<T>),
  Indirect(DrawIndirect<T>),
}

pub type Draw = DrawKind<BindVertexBuffers>;
pub type DrawManaged = DrawKind<BindVertexBuffersManaged>;

impl<T: BindVertexBuffersTrait> DrawKind<T> {
  pub fn new() -> T {
    T::default()
  }

  pub fn is_vertices(&self) -> bool {
    match self {
      DrawKind::Vertices(_) => true,
      _ => false,
    }
  }
  pub fn is_indexed(&self) -> bool {
    match self {
      DrawKind::Indexed(_) => true,
      _ => false,
    }
  }
  pub fn is_indirect(&self) -> bool {
    match self {
      DrawKind::Indirect(_) => true,
      _ => false,
    }
  }

  pub fn vertices(self) -> Option<DrawVertices<T>> {
    match self {
      DrawKind::Vertices(d) => Some(d),
      _ => None,
    }
  }
  pub fn indexed(self) -> Option<DrawIndexed<T>> {
    match self {
      DrawKind::Indexed(d) => Some(d),
      _ => None,
    }
  }
  pub fn indirect(self) -> Option<DrawIndirect<T>> {
    match self {
      DrawKind::Indirect(d) => Some(d),
      _ => None,
    }
  }
}

impl<T: BindVertexBuffersTrait> StreamPush for DrawKind<T> {
  fn enqueue(&self, cs: Stream) -> Stream {
    match self {
      DrawKind::Vertices(d) => cs.push(d),
      DrawKind::Indexed(d) => cs.push(d),
      DrawKind::Indirect(d) => cs.push(d),
    }
  }
}
