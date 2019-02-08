use vk;

/// Builder for a viewport state
///
/// Default initialized with
///
/// - primitive topology: vk::PRIMITIVE_TOPOLOGY_TRIANGLE_LIST
/// - primiteve restart: disabled
pub struct Builder {
  viewports: Vec<vk::Viewport>,
  scissor_rects: Vec<vk::Rect2D>,
  info: vk::PipelineViewportStateCreateInfo,
}

impl Builder {
  pub fn push_viewport(&mut self, vp: vk::Viewport) -> &mut Self {
    self.viewports.push(vp);
    self.update()
  }

  pub fn push_scissors_rect(&mut self, rect: vk::Rect2D) -> &mut Self {
    self.scissor_rects.push(rect);
    self.update()
  }

  fn update(&mut self) -> &mut Self {
    self.info.viewportCount = self.viewports.len() as u32;
    self.info.pViewports = self.viewports.as_ptr();
    self.info.scissorCount = self.scissor_rects.len() as u32;
    self.info.pScissors = self.scissor_rects.as_ptr();
    self
  }

  pub fn get(&self) -> &vk::PipelineViewportStateCreateInfo {
    &self.info
  }
}

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
