use super::Stream;
use super::StreamPush;
use vk;

/// Binds vertex buffers to command stream
#[derive(Debug)]
pub struct BindVertexBuffers {
  pub buffers: Vec<vk::Buffer>,
  pub offsets: Vec<vk::DeviceSize>,
  pub count: u32,
  pub pbuffers: *const vk::Buffer,
  pub poffsets: *const vk::DeviceSize,
}
/// Binds vertex buffers and issues draw call
#[derive(Default, Debug)]
pub struct DrawVertices {
  pub vertex_buffers: BindVertexBuffers,
  pub vertex_count: u32,
  pub instance_count: u32,
  pub first_vertex: u32,
  pub first_instance: u32,
}
/// Bind vertex buffers and issues an indexed draw call
#[derive(Default, Debug)]
pub struct DrawIndexed {
  pub vertex_buffers: BindVertexBuffers,
  pub index_count: u32,
  pub instance_count: u32,
  pub first_index: u32,
  pub vertex_offset: i32,
  pub first_instance: u32,

  pub index_buffer: vk::Buffer,
  pub index_buffer_offset: vk::DeviceSize,
  pub index_type: vk::IndexType,
}
/// Binds vertex buffers and issues an indirect draw call
#[derive(Default, Debug)]
pub struct DrawIndirect {
  pub vertex_buffers: BindVertexBuffers,
  pub count: u32,
  pub offset: vk::DeviceSize,
  pub stride: u32,
  pub buffer: vk::Buffer,

  pub index_buffer: Option<vk::Buffer>,
  pub index_offset: vk::DeviceSize,
  pub index_type: vk::IndexType,
}

#[derive(Debug)]
pub enum Draw {
  Vertices(DrawVertices),
  Indexed(DrawIndexed),
  Indirect(DrawIndirect),
}

impl Draw {
  pub fn new() -> BindVertexBuffers {
    BindVertexBuffers::default()
  }

  pub fn is_vertices(&self) -> bool {
    match self {
      Draw::Vertices(_) => true,
      _ => false,
    }
  }
  pub fn is_indexed(&self) -> bool {
    match self {
      Draw::Indexed(_) => true,
      _ => false,
    }
  }
  pub fn is_indirect(&self) -> bool {
    match self {
      Draw::Indirect(_) => true,
      _ => false,
    }
  }

  pub fn vertices(self) -> Option<DrawVertices> {
    match self {
      Draw::Vertices(d) => Some(d),
      _ => None,
    }
  }
  pub fn indexed(self) -> Option<DrawIndexed> {
    match self {
      Draw::Indexed(d) => Some(d),
      _ => None,
    }
  }
  pub fn indirect(self) -> Option<DrawIndirect> {
    match self {
      Draw::Indirect(d) => Some(d),
      _ => None,
    }
  }
}

impl StreamPush for Draw {
  fn enqueue(&self, cs: Stream) -> Stream {
    match self {
      Draw::Vertices(d) => cs.push(d),
      Draw::Indexed(d) => cs.push(d),
      Draw::Indirect(d) => cs.push(d),
    }
  }
}

impl Default for BindVertexBuffers {
  fn default() -> Self {
    Self {
      buffers: Default::default(),
      offsets: Default::default(),
      count: 0,
      pbuffers: std::ptr::null(),
      poffsets: std::ptr::null(),
    }
  }
}

impl BindVertexBuffers {
  pub fn push(mut self, buffer: vk::Buffer, offset: vk::DeviceSize) -> Self {
    self.buffers.push(buffer);
    self.offsets.push(offset);
    self
  }

  pub fn set_buffers(mut self, count: u32, buffers: *const vk::Buffer, offsets: *const vk::DeviceSize) -> Self {
    self.count = count;
    self.pbuffers = buffers;
    self.poffsets = offsets;
    self
  }

  pub fn vertices(mut self) -> DrawVertices {
    if !self.buffers.is_empty() {
      self.count = self.buffers.len() as u32;
      self.pbuffers = self.buffers.as_ptr();
      self.poffsets = self.offsets.as_ptr();
    }
    DrawVertices::new(self)
  }

  pub fn indexed(mut self, indices: vk::Buffer) -> DrawIndexed {
    if !self.buffers.is_empty() {
      self.count = self.buffers.len() as u32;
      self.pbuffers = self.buffers.as_ptr();
      self.poffsets = self.offsets.as_ptr();
    }
    DrawIndexed::new(self, indices)
  }

  pub fn indirect(mut self, buffer: vk::Buffer) -> DrawIndirect {
    if !self.buffers.is_empty() {
      self.count = self.buffers.len() as u32;
      self.pbuffers = self.buffers.as_ptr();
      self.poffsets = self.offsets.as_ptr();
    }
    DrawIndirect::new(self, buffer)
  }
}

impl StreamPush for BindVertexBuffers {
  fn enqueue(&self, cs: Stream) -> Stream {
    if self.count > 0 {
      vk::CmdBindVertexBuffers(
        cs.buffer,
        0,
        self.buffers.len() as u32,
        self.buffers.as_ptr(),
        self.offsets.as_ptr(),
      );
    }

    cs
  }
}

impl DrawVertices {
  /// Creates a new builder for normal drawing
  ///
  /// Default initializes:
  ///  - `index_count = 0`
  ///  - `instance_count = 1`
  ///  - `first_index = 0`
  ///  - `first_instance = 0`
  pub fn new(vertex_buffers: BindVertexBuffers) -> Self {
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

impl StreamPush for DrawVertices {
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

impl DrawIndexed {
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
  pub fn new(vertex_buffers: BindVertexBuffers, index_buffer: vk::Buffer) -> Self {
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

impl StreamPush for DrawIndexed {
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

impl DrawIndirect {
  /// Creates a new builder for indirect drawing
  ///
  /// Default initializes:
  ///  - `count = 0`
  ///  - `offset = 0`
  ///  - `stride = sizeof(vk::DrawIndirectCommand)`
  ///  - `index_buffer = None`
  pub fn new(vertex_buffers: BindVertexBuffers, buffer: vk::Buffer) -> Self {
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

impl StreamPush for DrawIndirect {
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
