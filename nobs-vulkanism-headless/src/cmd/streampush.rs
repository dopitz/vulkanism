use vk;

/// Allows to use [push](../stream/struct.Stream.html#method.push) on a [Stream](../stream/struct.Stream.html)
pub trait StreamPush {
  fn enqueue(&self, cb: vk::CommandBuffer);
}

/// Binds a pipeline to a command stream
pub struct BindPipeline {
  pub bindpoint: vk::PipelineBindPoint,
  pub pipeline: vk::Pipeline,
}

impl BindPipeline {
  pub fn new(bindpoint: vk::PipelineBindPoint, pipeline: vk::Pipeline) -> Self {
    Self { bindpoint, pipeline }
  }

  pub fn compute(pipeline: vk::Pipeline) -> Self {
    Self::new(vk::PIPELINE_BIND_POINT_COMPUTE, pipeline)
  }

  pub fn graphics(pipeline: vk::Pipeline) -> Self {
    Self::new(vk::PIPELINE_BIND_POINT_GRAPHICS, pipeline)
  }

  pub fn raytrace(pipeline: vk::Pipeline) -> Self {
    Self::new(vk::BUFFER_USAGE_RAY_TRACING_BIT_NV, pipeline)
  }
}

impl StreamPush for BindPipeline {
  fn enqueue(&self, cb: vk::CommandBuffer) {
    vk::CmdBindPipeline(cb, self.bindpoint, self.pipeline);
  }
}

/// Binds a descriptor set to a command stream
pub struct BindDset {
  pub bindpoint: vk::PipelineBindPoint,
  pub layout: vk::PipelineLayout,
  pub index: u32,
  pub dset: vk::DescriptorSet,
}

impl BindDset {
  pub fn new(bindpoint: vk::PipelineBindPoint, layout: vk::PipelineLayout, index: u32, dset: vk::DescriptorSet) -> Self {
    Self {
      bindpoint,
      layout,
      index,
      dset,
    }
  }
}

impl StreamPush for BindDset {
  fn enqueue(&self, cb: vk::CommandBuffer) {
    vk::CmdBindDescriptorSets(cb, self.bindpoint, self.layout, self.index, 1, &self.dset, 0, std::ptr::null());
  }
}

/// Issues a dispatch call for compute pipelines in a command stream
pub enum Dispatch {
  Base(DispatchBase),
  Indirect(DispatchIndirect),
}
pub struct DispatchBase {
  pub x: u32,
  pub y: u32,
  pub z: u32,
}
pub struct DispatchIndirect {
  pub offset: vk::DeviceSize,
  pub buffer: vk::Buffer,
}

impl Dispatch {
  pub fn x(x: u32) -> Self {
    Self::xyz(x, 1, 1)
  }
  pub fn xy(x: u32, y: u32) -> Self {
    Self::xyz(x, y, 1)
  }
  pub fn xyz(x: u32, y: u32, z: u32) -> Self {
    Dispatch::Base(DispatchBase { x, y: y, z: z })
  }

  pub fn indirect(buffer: vk::Buffer) -> Self {
    Self::indirect_offset(buffer, 0)
  }
  pub fn indirect_offset(buffer: vk::Buffer, offset: vk::DeviceSize) -> Self {
    Dispatch::Indirect(DispatchIndirect { offset, buffer })
  }
}

impl StreamPush for Dispatch {
  fn enqueue(&self, cb: vk::CommandBuffer) {
    match self {
      Dispatch::Base(d) => vk::CmdDispatch(cb, d.x, d.y, d.z),
      Dispatch::Indirect(d) => vk::CmdDispatchIndirect(cb, d.buffer, d.offset),
    }
  }
}

/// Binds vertex buffers to command stream
#[derive(Default, Clone)]
pub struct BindVertexBuffers {
  pub buffers: Vec<vk::Buffer>,
  pub offsets: Vec<vk::DeviceSize>,
}
/// Binds vertex buffers and issues draw call
#[derive(Default)]
pub struct DrawVertices {
  pub vertex_buffers: BindVertexBuffers,
  pub vertex_count: u32,
  pub instance_count: u32,
  pub first_vertex: u32,
  pub first_instance: u32,
}
/// Bind vertex buffers and issues an indexed draw call
#[derive(Default)]
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
#[derive(Default)]
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

pub type Draw = BindVertexBuffers;

impl BindVertexBuffers {
  pub fn push(mut self, buffer: vk::Buffer, offset: vk::DeviceSize) -> Self {
    self.buffers.push(buffer);
    self.offsets.push(offset);
    self
  }

  pub fn vertices(self) -> DrawVertices {
    DrawVertices::new(self)
  }

  pub fn indexed(self, indices: vk::Buffer) -> DrawIndexed {
    DrawIndexed::new(self, indices)
  }

  pub fn indirect(self, buffer: vk::Buffer) -> DrawIndirect {
    DrawIndirect::new(self, buffer)
  }
}

impl StreamPush for BindVertexBuffers {
  fn enqueue(&self, cb: vk::CommandBuffer) {
    if !self.buffers.is_empty() {
      vk::CmdBindVertexBuffers(cb, 0, self.buffers.len() as u32, self.buffers.as_ptr(), self.offsets.as_ptr());
    }
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
  fn enqueue(&self, cb: vk::CommandBuffer) {
    self.vertex_buffers.enqueue(cb);
    vk::CmdDraw(cb, self.vertex_count, self.instance_count, self.first_vertex, self.first_instance);
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
  fn enqueue(&self, cb: vk::CommandBuffer) {
    self.vertex_buffers.enqueue(cb);
    vk::CmdBindIndexBuffer(cb, self.index_buffer, self.index_buffer_offset, self.index_type);
    vk::CmdDrawIndexed(
      cb,
      self.index_count,
      self.instance_count,
      self.first_index,
      self.vertex_offset,
      self.first_instance,
    );
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
  fn enqueue(&self, cb: vk::CommandBuffer) {
    self.vertex_buffers.enqueue(cb);
    if let Some(indices) = self.index_buffer {
      vk::CmdBindIndexBuffer(cb, indices, self.index_offset, self.index_type);
      vk::CmdDrawIndexedIndirect(cb, self.buffer, self.offset, self.count, self.stride);
    } else {
      vk::CmdDrawIndirect(cb, self.buffer, self.offset, self.count, self.stride);
    }
  }
}

/// Copies memory from one buffer to another
pub struct BufferCopy {
  pub src: vk::Buffer,
  pub dst: vk::Buffer,
  pub region: vk::BufferCopy,
}

impl Default for BufferCopy {
  fn default() -> Self {
    BufferCopy {
      src: vk::NULL_HANDLE,
      dst: vk::NULL_HANDLE,
      region: vk::BufferCopy {
        size: 0,
        dstOffset: 0,
        srcOffset: 0,
      },
    }
  }
}

impl BufferCopy {
  pub fn new(src: vk::Buffer, dst: vk::Buffer, region: vk::BufferCopy) -> BufferCopy {
    BufferCopy { src, dst, region }
  }

  pub fn from(&mut self, src: vk::Buffer) -> &mut Self {
    self.src = src;
    self
  }
  pub fn from_offset(&mut self, src: vk::Buffer, offset: vk::DeviceSize) -> &mut Self {
    self.src = src;
    self.region.srcOffset = offset;
    self
  }

  pub fn to(&mut self, src: vk::Buffer) -> &mut Self {
    self.src = src;
    self
  }
  pub fn to_offset(&mut self, dst: vk::Buffer, offset: vk::DeviceSize) -> &mut Self {
    self.dst = dst;
    self.region.dstOffset = offset;
    self
  }

  pub fn size(&mut self, size: vk::DeviceSize) -> &mut Self {
    self.region.size = size;
    self
  }
}

impl StreamPush for BufferCopy {
  fn enqueue(&self, cb: vk::CommandBuffer) {
    vk::CmdCopyBuffer(cb, self.src, self.dst, 1, &self.region);
  }
}

/// Conservatively gets pipeline stage flags from acces flags
fn get_stages_from_access(access: vk::AccessFlags) -> vk::PipelineStageFlags {
  let mut res = 0;

  if (access & vk::ACCESS_INDIRECT_COMMAND_READ_BIT) != 0 {
    res |= vk::PIPELINE_STAGE_DRAW_INDIRECT_BIT;
  }
  if (access & vk::ACCESS_INDEX_READ_BIT) != 0 {
    res |= vk::PIPELINE_STAGE_VERTEX_INPUT_BIT;
  }
  if (access & vk::ACCESS_VERTEX_ATTRIBUTE_READ_BIT) != 0 {
    res |= vk::PIPELINE_STAGE_VERTEX_INPUT_BIT;
  }
  if (access & vk::ACCESS_INPUT_ATTACHMENT_READ_BIT) != 0 {
    res |= vk::PIPELINE_STAGE_FRAGMENT_SHADER_BIT;
  }
  if (access & (vk::ACCESS_UNIFORM_READ_BIT | vk::ACCESS_SHADER_READ_BIT | vk::ACCESS_SHADER_WRITE_BIT)) != 0 {
    res |= vk::PIPELINE_STAGE_VERTEX_SHADER_BIT
      | vk::PIPELINE_STAGE_TESSELLATION_CONTROL_SHADER_BIT
      | vk::PIPELINE_STAGE_TESSELLATION_EVALUATION_SHADER_BIT
      | vk::PIPELINE_STAGE_GEOMETRY_SHADER_BIT
      | vk::PIPELINE_STAGE_FRAGMENT_SHADER_BIT
      | vk::PIPELINE_STAGE_COMPUTE_SHADER_BIT;
  }
  if (access & (vk::ACCESS_COLOR_ATTACHMENT_READ_BIT | vk::ACCESS_COLOR_ATTACHMENT_WRITE_BIT)) != 0 {
    res |= vk::PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT;
  }
  if (access & (vk::ACCESS_DEPTH_STENCIL_ATTACHMENT_READ_BIT | vk::ACCESS_DEPTH_STENCIL_ATTACHMENT_WRITE_BIT)) != 0 {
    res |= vk::PIPELINE_STAGE_EARLY_FRAGMENT_TESTS_BIT | vk::PIPELINE_STAGE_LATE_FRAGMENT_TESTS_BIT;
  }
  if (access & (vk::ACCESS_TRANSFER_READ_BIT | vk::ACCESS_TRANSFER_WRITE_BIT)) != 0 {
    res |= vk::PIPELINE_STAGE_TRANSFER_BIT;
  }
  if (access & (vk::ACCESS_HOST_READ_BIT | vk::ACCESS_HOST_WRITE_BIT)) != 0 {
    res |= vk::PIPELINE_STAGE_HOST_BIT;
  }
  if (access & vk::ACCESS_COLOR_ATTACHMENT_READ_NONCOHERENT_BIT_EXT) != 0 {
    res |= vk::PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT;
  }
  if (access & (vk::ACCESS_COMMAND_PROCESS_READ_BIT_NVX | vk::ACCESS_COMMAND_PROCESS_WRITE_BIT_NVX)) != 0 {
    res |= vk::PIPELINE_STAGE_COMMAND_PROCESS_BIT_NVX;
  }
  // case vk::ACCESS_MEMORY_READ_BIT: return 0;
  // case vk::ACCESS_MEMORY_WRITE_BIT: return 0;

  res
}

/// Creates a read after write protecting barrier for an image resource in a command stream
pub struct ImageBarrier {
  pub src_stages: vk::PipelineStageFlags,
  pub dst_stages: vk::PipelineStageFlags,
  pub barrier: vk::ImageMemoryBarrier,
}

impl ImageBarrier {
  pub fn new(img: vk::Image) -> Self {
    ImageBarrier {
      src_stages: vk::PIPELINE_STAGE_TOP_OF_PIPE_BIT,
      dst_stages: 0,
      barrier: vk::ImageMemoryBarrier {
        sType: vk::STRUCTURE_TYPE_IMAGE_MEMORY_BARRIER,
        pNext: std::ptr::null(),
        srcAccessMask: 0,
        dstAccessMask: 0,
        oldLayout: vk::IMAGE_LAYOUT_UNDEFINED,
        newLayout: vk::IMAGE_LAYOUT_UNDEFINED,
        srcQueueFamilyIndex: vk::QUEUE_FAMILY_IGNORED,
        dstQueueFamilyIndex: vk::QUEUE_FAMILY_IGNORED,
        image: img,
        subresourceRange: vk::ImageSubresourceRange {
          aspectMask: vk::IMAGE_ASPECT_COLOR_BIT,
          baseMipLevel: 0,
          levelCount: 1,
          baseArrayLayer: 0,
          layerCount: 1,
        },
      },
    }
  }

  pub fn from(&mut self, layout: vk::ImageLayout, access: vk::AccessFlags) -> &mut Self {
    self.from_stages(layout, access, get_stages_from_access(access))
  }
  pub fn from_stages(&mut self, layout: vk::ImageLayout, access: vk::AccessFlags, stages: vk::PipelineStageFlags) -> &mut Self {
    self.src_stages = stages;
    self.barrier.oldLayout = layout;
    self.barrier.srcAccessMask = access;
    self
  }

  pub fn to(&mut self, layout: vk::ImageLayout, access: vk::AccessFlags) -> &mut Self {
    self.to_stages(layout, access, get_stages_from_access(access))
  }
  pub fn to_stages(&mut self, layout: vk::ImageLayout, access: vk::AccessFlags, stages: vk::PipelineStageFlags) -> &mut Self {
    self.dst_stages = stages;
    self.barrier.newLayout = layout;
    self.barrier.dstAccessMask = access;
    self
  }

  pub fn aspect_mask(&mut self, aspect: vk::ImageAspectFlags) -> &mut Self {
    self.barrier.subresourceRange.aspectMask = aspect;
    self
  }
  pub fn mip_level(&mut self, base_level: u32, count: u32) -> &mut Self {
    self.barrier.subresourceRange.baseMipLevel = base_level;
    self.barrier.subresourceRange.levelCount = count;
    self
  }
  pub fn array_layer(&mut self, base_layer: u32, count: u32) -> &mut Self {
    self.barrier.subresourceRange.baseArrayLayer = base_layer;
    self.barrier.subresourceRange.layerCount = count;
    self
  }
  pub fn subresource(&mut self, subresource: vk::ImageSubresourceRange) -> &mut Self {
    self.barrier.subresourceRange = subresource;
    self
  }
}

impl StreamPush for ImageBarrier {
  fn enqueue(&self, cb: vk::CommandBuffer) {
    vk::CmdPipelineBarrier(
      cb,
      self.src_stages,
      self.dst_stages,
      0,
      0,
      std::ptr::null(),
      0,
      std::ptr::null(),
      1,
      &self.barrier,
    );
  }
}

/// Creates a read after write protecting barrier for a buffer resource in a command stream
pub struct BufferBarrier {
  pub src_stages: vk::PipelineStageFlags,
  pub dst_stages: vk::PipelineStageFlags,
  pub barrier: vk::BufferMemoryBarrier,
}

impl BufferBarrier {
  pub fn new(buf: vk::Buffer) -> Self {
    Self::with_size(buf, 0, vk::WHOLE_SIZE)
  }
  pub fn with_size(buf: vk::Buffer, offset: vk::DeviceSize, size: vk::DeviceSize) -> Self {
    Self {
      src_stages: vk::PIPELINE_STAGE_TOP_OF_PIPE_BIT,
      dst_stages: 0,
      barrier: vk::BufferMemoryBarrier {
        sType: vk::STRUCTURE_TYPE_BUFFER_MEMORY_BARRIER,
        pNext: std::ptr::null(),
        srcAccessMask: 0,
        dstAccessMask: 0,
        srcQueueFamilyIndex: vk::QUEUE_FAMILY_IGNORED,
        dstQueueFamilyIndex: vk::QUEUE_FAMILY_IGNORED,
        buffer: buf,
        offset: offset,
        size: size,
      },
    }
  }

  pub fn from(&mut self, access: vk::AccessFlags) -> &mut Self {
    self.from_stages(access, get_stages_from_access(access))
  }
  pub fn from_stages(&mut self, access: vk::AccessFlags, stages: vk::PipelineStageFlags) -> &mut Self {
    self.src_stages = stages;
    self.barrier.srcAccessMask = access;
    self
  }

  pub fn to(&mut self, access: vk::AccessFlags) -> &mut Self {
    self.to_stages(access, get_stages_from_access(access))
  }
  pub fn to_stages(&mut self, access: vk::AccessFlags, stages: vk::PipelineStageFlags) -> &mut Self {
    self.dst_stages = stages;
    self.barrier.dstAccessMask = access;
    self
  }
}

impl StreamPush for BufferBarrier {
  fn enqueue(&self, cb: vk::CommandBuffer) {
    vk::CmdPipelineBarrier(
      cb,
      self.src_stages,
      self.dst_stages,
      0,
      0,
      std::ptr::null(),
      1,
      &self.barrier,
      0,
      std::ptr::null(),
    );
  }
}

/// Clear Color render target
#[derive(Clone, Copy)]
pub struct ClearColorImage {
  pub image: vk::Image,
  pub layout: vk::ImageLayout,
  pub clear: vk::ClearColorValue,
  pub subresource: vk::ImageSubresourceRange,
}

impl ClearColorImage {
  pub fn new(image: vk::Image) -> Self {
    Self {
      image,
      layout: vk::IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL,
      clear: vk::ClearColorValue { int32: [0, 0, 0, 0] },
      subresource: vk::ImageSubresourceRange {
        aspectMask: vk::IMAGE_ASPECT_COLOR_BIT,
        baseMipLevel: 0,
        levelCount: 1,
        baseArrayLayer: 0,
        layerCount: 1,
      },
    }
  }

  pub fn layout(&mut self, layout: vk::ImageLayout) -> &mut Self {
    self.layout = layout;
    self
  }

  pub fn clear(&mut self, clear: vk::ClearColorValue) -> &mut Self {
    self.clear = clear;
    self
  }

  pub fn subresource(&mut self, subresource: vk::ImageSubresourceRange) -> &mut Self {
    self.subresource = subresource;
    self
  }
}

impl StreamPush for ClearColorImage {
  fn enqueue(&self, cb: vk::CommandBuffer) {
    vk::CmdClearColorImage(cb, self.image, self.layout, &self.clear, 1, &self.subresource);
  }
}

/// Begins a render pass
#[derive(Clone, Copy)]
pub struct RenderpassBegin {
  pub info: vk::RenderPassBeginInfo,
  pub contents: vk::SubpassContents,
}

impl RenderpassBegin {
  pub fn new(pass: vk::RenderPass, framebuffer: vk::Framebuffer) -> RenderpassBegin {
    Self {
      info: vk::RenderPassBeginInfo {
        sType: vk::STRUCTURE_TYPE_RENDER_PASS_BEGIN_INFO,
        pNext: std::ptr::null(),
        renderPass: pass,
        framebuffer: framebuffer,
        renderArea: vk::Rect2D {
          offset: vk::Offset2D { x: 0, y: 0 },
          extent: vk::Extent2D { width: 0, height: 0 },
        },
        clearValueCount: 0,
        pClearValues: std::ptr::null(),
      },
      contents: vk::SUBPASS_CONTENTS_INLINE,
    }
  }

  pub fn contents(&mut self, contents: vk::SubpassContents) -> &mut Self {
    self.contents = contents;
    self
  }

  pub fn offset(&mut self, offset: vk::Offset2D) -> &mut Self {
    self.info.renderArea.offset = offset;
    self
  }
  pub fn extent(&mut self, extent: vk::Extent2D) -> &mut Self {
    self.info.renderArea.extent = extent;
    self
  }
  pub fn area(&mut self, area: vk::Rect2D) -> &mut Self {
    self.info.renderArea = area;
    self
  }

  pub fn clear(&mut self, clear: &[vk::ClearValue]) -> &mut Self {
    self.info.clearValueCount = clear.len() as u32;
    self.info.pClearValues = clear.as_ptr();
    self
  }
}

impl StreamPush for RenderpassBegin {
  fn enqueue(&self, cb: vk::CommandBuffer) {
    vk::CmdBeginRenderPass(cb, &self.info, self.contents);
  }
}

/// Ends a render pass
pub struct RenderpassEnd {}

impl StreamPush for RenderpassEnd {
  fn enqueue(&self, cb: vk::CommandBuffer) {
    vk::CmdEndRenderPass(cb);
  }
}

/// Blits src to dst
#[derive(Clone, Copy)]
pub struct Blit {
  region: vk::ImageBlit,
  src: vk::Image,
  dst: vk::Image,
  filter: vk::Filter,
}

impl Blit {
  pub fn new() -> Blit {
    Self {
      region: vk::ImageBlit {
        srcSubresource: vk::ImageSubresourceLayers {
          aspectMask: vk::IMAGE_ASPECT_COLOR_BIT,
          mipLevel: 0,
          baseArrayLayer: 0,
          layerCount: 1,
        },
        srcOffsets: [vk::Offset3D { x: 0, y: 0, z: 0 }, vk::Offset3D { x: 0, y: 0, z: 0 }],
        dstSubresource: vk::ImageSubresourceLayers {
          aspectMask: vk::IMAGE_ASPECT_COLOR_BIT,
          mipLevel: 0,
          baseArrayLayer: 0,
          layerCount: 1,
        },
        dstOffsets: [vk::Offset3D { x: 0, y: 0, z: 0 }, vk::Offset3D { x: 0, y: 0, z: 0 }],
      },
      src: vk::NULL_HANDLE,
      dst: vk::NULL_HANDLE,
      filter: vk::FILTER_LINEAR,
    }
  }

  pub fn src(&mut self, img: vk::Image) -> &mut Self {
    self.src = img;
    self
  }

  pub fn src_subresource(&mut self, subresource: vk::ImageSubresourceLayers) -> &mut Self {
    self.region.srcSubresource = subresource;
    self
  }

  pub fn src_offset_begin(&mut self, x: i32, y: i32, z: i32) -> &mut Self {
    self.region.srcOffsets[0] = vk::Offset3D { x, y, z };
    self
  }

  pub fn src_offset_end(&mut self, x: i32, y: i32, z: i32) -> &mut Self {
    self.region.srcOffsets[1] = vk::Offset3D { x, y, z };
    self
  }

  pub fn dst(&mut self, img: vk::Image) -> &mut Self {
    self.dst = img;
    self
  }

  pub fn dst_subresource(&mut self, subresource: vk::ImageSubresourceLayers) -> &mut Self {
    self.region.dstSubresource = subresource;
    self
  }

  pub fn dst_offset_begin(&mut self, x: i32, y: i32, z: i32) -> &mut Self {
    self.region.dstOffsets[0] = vk::Offset3D { x, y, z };
    self
  }

  pub fn dst_offset_end(&mut self, x: i32, y: i32, z: i32) -> &mut Self {
    self.region.dstOffsets[1] = vk::Offset3D { x, y, z };
    self
  }

  pub fn filter(&mut self, filter: vk::Filter) -> &mut Self {
    self.filter = filter;
    self
  }
}

impl StreamPush for Blit {
  fn enqueue(&self, cb: vk::CommandBuffer) {
    ImageBarrier::new(self.src)
      .to(vk::IMAGE_LAYOUT_TRANSFER_SRC_OPTIMAL, vk::ACCESS_COLOR_ATTACHMENT_WRITE_BIT)
      .enqueue(cb);
    ImageBarrier::new(self.dst)
      .to(vk::IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL, vk::ACCESS_TRANSFER_WRITE_BIT)
      .enqueue(cb);
    vk::CmdBlitImage(
      cb,
      self.src,
      vk::IMAGE_LAYOUT_TRANSFER_SRC_OPTIMAL,
      self.dst,
      vk::IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL,
      1,
      &self.region,
      self.filter,
    );
    ImageBarrier::new(self.dst)
      .to(vk::IMAGE_LAYOUT_PRESENT_SRC_KHR, vk::ACCESS_COLOR_ATTACHMENT_READ_BIT)
      .enqueue(cb);
  }
}

/// Sets a viewport for the command stream
#[derive(Clone, Copy)]
pub struct Viewport {
  pub vp: vk::Viewport,
}

impl Viewport {
  pub fn with_size(width: f32, height: f32) -> Self {
    Self {
      vp: vk::Viewport {
        x: 0.0,
        y: 0.0,
        width,
        height,
        minDepth: 0.0,
        maxDepth: 1.0,
      },
    }
  }

  pub fn with_extent(extent: vk::Extent2D) -> Self {
    Self::with_size(extent.width as f32, extent.height as f32)
  }

  pub fn offset(mut self, x: f32, y: f32) -> Self {
    self.vp.x = x;
    self.vp.y = y;
    self
  }

  pub fn depth(mut self, mindepth: f32, maxdepth: f32) -> Self {
    self.vp.minDepth = mindepth;
    self.vp.maxDepth = maxdepth;
    self
  }
}

impl StreamPush for Viewport {
  fn enqueue(&self, cb: vk::CommandBuffer) {
    vk::CmdSetViewport(cb, 0, 1, &self.vp);
  }
}

/// Sets a scissor rect for the command stream
#[derive(Clone, Copy)]
pub struct Scissor {
  pub rect: vk::Rect2D,
}

impl Scissor {
  pub fn with_size(width: u32, height: u32) -> Self {
    Self::with_extent(vk::Extent2D { width, height })
  }

  pub fn with_extent(extent: vk::Extent2D) -> Self {
    Self::with_offset(vk::Offset2D { x: 0, y: 0 }, extent)
  }

  pub fn with_offset(offset: vk::Offset2D, extent: vk::Extent2D) -> Self {
    Self {
      rect: vk::Rect2D { offset, extent },
    }
  }
}

impl StreamPush for Scissor {
  fn enqueue(&self, cb: vk::CommandBuffer) {
    vk::CmdSetScissor(cb, 0, 1, &self.rect);
  }
}
