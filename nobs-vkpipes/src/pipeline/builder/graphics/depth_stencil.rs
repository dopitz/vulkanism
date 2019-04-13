use vk;

/// Builder for a depth stencil state
///
/// Default initialized with
///
/// - depth test: enabled
/// - depth write: enabled
/// - depth compare: `vk::COMPARE_OP_LESS`
/// - depth bounds test: disabled
/// - min depth bounds: `0`
/// - max depth bounds: `1`
/// - stencil test: disabled
/// - front: uninitialized
/// - back: uninitialized
pub struct Builder {
  pub info: vk::PipelineDepthStencilStateCreateInfo,
}

vk_builder!(vk::PipelineDepthStencilStateCreateInfo, Builder);

impl Default for Builder {
  fn default() -> Builder {
    Builder {
      info: vk::PipelineDepthStencilStateCreateInfo {
        sType: vk::STRUCTURE_TYPE_PIPELINE_DEPTH_STENCIL_STATE_CREATE_INFO,
        pNext: std::ptr::null(),
        flags: 0,
        depthTestEnable: vk::TRUE,
        depthWriteEnable: vk::TRUE,
        depthCompareOp: vk::COMPARE_OP_LESS_OR_EQUAL,
        depthBoundsTestEnable: vk::FALSE,
        minDepthBounds: 0.0f32,
        maxDepthBounds: 1.0f32,
        stencilTestEnable: vk::FALSE,
        front: unsafe { std::mem::zeroed() },
        back: unsafe { std::mem::zeroed() },
      },
    }
  }
}

impl Builder {
  pub fn depth_test_enable(mut self, enable: vk::Bool32) -> Self {
    self.info.depthTestEnable = enable;
    self
  }
  pub fn depth_write_enable(mut self, enable: vk::Bool32) -> Self {
    self.info.depthWriteEnable = enable;
    self
  }
  pub fn depth_compare(mut self, op: vk::CompareOp) -> Self {
    self.info.depthCompareOp = op;
    self
  }
  pub fn depth_bounds_test_enable(mut self, enable: vk::Bool32) -> Self {
    self.info.depthBoundsTestEnable = enable;
    self
  }
  pub fn min_depth_bounds(mut self, min: f32) -> Self {
    self.info.minDepthBounds = min;
    self
  }
  pub fn max_depth_bounds(mut self, max: f32) -> Self {
    self.info.maxDepthBounds = max;
    self
  }
  pub fn stencil_test_enable(mut self, enable: vk::Bool32) -> Self {
    self.info.stencilTestEnable = enable;
    self
  }
  pub fn front(mut self, front: vk::StencilOpState) -> Self {
    self.info.front = front;
    self
  }
  pub fn back(mut self, back: vk::StencilOpState) -> Self {
    self.info.back = back;
    self
  }
}
