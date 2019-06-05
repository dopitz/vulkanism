use super::Stream;
use super::StreamPush;
use vk;

/// Binds a pipeline to a command stream
#[derive(Debug, Clone, Copy)]
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
  fn enqueue(&self, cs: Stream) -> Stream {
    vk::CmdBindPipeline(cs.buffer, self.bindpoint, self.pipeline);
    cs
  }
}

/// Binds a descriptor set to a command stream
#[derive(Debug)]
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
  fn enqueue(&self, cs: Stream) -> Stream {
    vk::CmdBindDescriptorSets(
      cs.buffer,
      self.bindpoint,
      self.layout,
      self.index,
      1,
      &self.dset,
      0,
      std::ptr::null(),
    );
    cs
  }
}

/// Sets a viewport for the command stream
#[derive(Debug, Clone, Copy)]
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
  fn enqueue(&self, cs: Stream) -> Stream {
    vk::CmdSetViewport(cs.buffer, 0, 1, &self.vp);
    cs
  }
}

/// Sets a scissor rect for the command stream
#[derive(Debug, Clone, Copy)]
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
  fn enqueue(&self, cs: Stream) -> Stream {
    vk::CmdSetScissor(cs.buffer, 0, 1, &self.rect);
    cs
  }
}
