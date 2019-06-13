use super::bindvertexbuffer::BindVertexBuffersTrait;
use super::BindVertexBuffers;
use super::BindVertexBuffersManaged;
use super::DrawIndexed;
use super::DrawIndirect;
use super::DrawVertices;
use crate::cmd::stream::*;

#[derive(Debug, Clone, Copy)]
pub enum DrawKind {
  Vertices(DrawVertices),
  Indexed(DrawIndexed),
  Indirect(DrawIndirect),
}

impl Default for DrawKind {
  fn default() -> Self {
    DrawKind::Vertices(Default::default())
  }
}

impl DrawKind {
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

  pub fn vertices(&self) -> Option<&DrawVertices> {
    match self {
      DrawKind::Vertices(d) => Some(d),
      _ => None,
    }
  }
  pub fn indexed(&self) -> Option<&DrawIndexed> {
    match self {
      DrawKind::Indexed(d) => Some(d),
      _ => None,
    }
  }
  pub fn indirect(&self) -> Option<&DrawIndirect> {
    match self {
      DrawKind::Indirect(d) => Some(d),
      _ => None,
    }
  }
}

impl StreamPush for DrawKind {
  fn enqueue(&self, cs: CmdBuffer) -> CmdBuffer {
    match self {
      DrawKind::Vertices(d) => cs.push(d),
      DrawKind::Indexed(d) => cs.push(d),
      DrawKind::Indirect(d) => cs.push(d),
    }
  }
}

#[derive(Default, Debug, Clone, Copy)]
pub struct Draw {
  pub vbs: BindVertexBuffers,
  pub draw: DrawKind,
}

impl Draw {
  pub fn new(vbs: BindVertexBuffers, draw: DrawKind) -> Self {
    Self { vbs, draw }
  }
}

impl StreamPush for Draw {
  fn enqueue(&self, cs: CmdBuffer) -> CmdBuffer {
    cs.push(&self.vbs).push(&self.draw)
  }
}

#[derive(Default, Debug, Clone)]
pub struct DrawManaged {
  pub vbs: BindVertexBuffersManaged,
  pub draw: DrawKind,
}

impl DrawManaged {
  pub fn new(vbs: BindVertexBuffersManaged, draw: DrawKind) -> Self {
    Self { vbs, draw }
  }
}

impl StreamPush for DrawManaged {
  fn enqueue(&self, cs: CmdBuffer) -> CmdBuffer {
    cs.push(&self.vbs).push(&self.draw)
  }
}

impl From<DrawVertices> for DrawKind {
  fn from(vertices: DrawVertices) -> Self {
    DrawKind::Vertices(vertices)
  }
}
impl From<DrawIndexed> for DrawKind {
  fn from(indexed: DrawIndexed) -> Self {
    DrawKind::Indexed(indexed)
  }
}
impl From<DrawIndirect> for DrawKind {
  fn from(indirect: DrawIndirect) -> Self {
    DrawKind::Indirect(indirect)
  }
}
