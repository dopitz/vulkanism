use vk;

/// Builder for a viewport state
///
/// Default initialized with
///  - viewports: none
///  - scissor_rects: none
pub struct Builder {
  viewports: Vec<vk::Viewport>,
  scissor_rects: Vec<vk::Rect2D>,
  info: vk::PipelineViewportStateCreateInfo,
}

vk_builder!(vk::PipelineViewportStateCreateInfo, Builder);

impl Default for Builder {
  fn default() -> Builder {
    Builder {
      viewports: Default::default(),
      scissor_rects: Default::default(),
      info: vk::PipelineViewportStateCreateInfo {
        sType: vk::STRUCTURE_TYPE_PIPELINE_VIEWPORT_STATE_CREATE_INFO,
        pNext: std::ptr::null(),
        flags: 0,
        viewportCount: 0,
        pViewports: std::ptr::null(),
        scissorCount: 0,
        pScissors: std::ptr::null(),
      },
    }
  }
}

impl Builder {
  pub fn push_viewport(mut self, vp: vk::Viewport) -> Self {
    self.viewports.push(vp);
    self.info.viewportCount = self.viewports.len() as u32;
    self.info.pViewports = self.viewports.as_ptr();
    self
  }
  pub fn push_scissors_rect(mut self, rect: vk::Rect2D) -> Self {
    self.scissor_rects.push(rect);
    self.info.scissorCount = self.scissor_rects.len() as u32;
    self.info.pScissors = self.scissor_rects.as_ptr();
    self
  }
}
