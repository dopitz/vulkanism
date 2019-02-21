use super::Stream;
use super::StreamPush;
use vk;

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

  pub fn to_color_attachment(img: vk::Image) -> Self {
    Self::new(img).to(vk::IMAGE_LAYOUT_COLOR_ATTACHMENT_OPTIMAL, vk::ACCESS_COLOR_ATTACHMENT_WRITE_BIT)
  }
  pub fn to_present(img: vk::Image) -> Self {
    Self::new(img).to(vk::IMAGE_LAYOUT_PRESENT_SRC_KHR, vk::ACCESS_COLOR_ATTACHMENT_READ_BIT)
  }
  pub fn to_transfer_src(img: vk::Image) -> Self {
    Self::new(img).to(vk::IMAGE_LAYOUT_TRANSFER_SRC_OPTIMAL, vk::ACCESS_COLOR_ATTACHMENT_WRITE_BIT)
  }
  pub fn to_transfer_dst(img: vk::Image) -> Self {
    Self::new(img).to(vk::IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL, vk::ACCESS_TRANSFER_WRITE_BIT)
  }

  pub fn from(self, layout: vk::ImageLayout, access: vk::AccessFlags) -> Self {
    self.from_stages(layout, access, get_stages_from_access(access))
  }
  pub fn from_stages(mut self, layout: vk::ImageLayout, access: vk::AccessFlags, stages: vk::PipelineStageFlags) -> Self {
    self.src_stages = stages;
    self.barrier.oldLayout = layout;
    self.barrier.srcAccessMask = access;
    self
  }

  pub fn to(self, layout: vk::ImageLayout, access: vk::AccessFlags) -> Self {
    self.to_stages(layout, access, get_stages_from_access(access))
  }
  pub fn to_stages(mut self, layout: vk::ImageLayout, access: vk::AccessFlags, stages: vk::PipelineStageFlags) -> Self {
    self.dst_stages = stages;
    self.barrier.newLayout = layout;
    self.barrier.dstAccessMask = access;
    self
  }

  pub fn aspect_mask(mut self, aspect: vk::ImageAspectFlags) -> Self {
    self.barrier.subresourceRange.aspectMask = aspect;
    self
  }
  pub fn mip_level(mut self, base_level: u32, count: u32) -> Self {
    self.barrier.subresourceRange.baseMipLevel = base_level;
    self.barrier.subresourceRange.levelCount = count;
    self
  }
  pub fn array_layer(mut self, base_layer: u32, count: u32) -> Self {
    self.barrier.subresourceRange.baseArrayLayer = base_layer;
    self.barrier.subresourceRange.layerCount = count;
    self
  }
  pub fn subresource(mut self, subresource: vk::ImageSubresourceRange) -> Self {
    self.barrier.subresourceRange = subresource;
    self
  }
}

impl StreamPush for ImageBarrier {
  fn enqueue(&self, cs: Stream) -> Stream {
    vk::CmdPipelineBarrier(
      cs.buffer,
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
    cs
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

  pub fn from(self, access: vk::AccessFlags) -> Self {
    self.from_stages(access, get_stages_from_access(access))
  }
  pub fn from_stages(mut self, access: vk::AccessFlags, stages: vk::PipelineStageFlags) -> Self {
    self.src_stages = stages;
    self.barrier.srcAccessMask = access;
    self
  }

  pub fn to(self, access: vk::AccessFlags) -> Self {
    self.to_stages(access, get_stages_from_access(access))
  }
  pub fn to_stages(mut self, access: vk::AccessFlags, stages: vk::PipelineStageFlags) -> Self {
    self.dst_stages = stages;
    self.barrier.dstAccessMask = access;
    self
  }
}

impl StreamPush for BufferBarrier {
  fn enqueue(&self, cs: Stream) -> Stream {
    vk::CmdPipelineBarrier(
      cs.buffer,
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
    cs
  }
}
