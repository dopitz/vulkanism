use vk;

/// Builder for a tesselation state
///
/// Default initialized with
///
/// - path control points: `0`
pub struct Builder {
  pub info: vk::PipelineTessellationStateCreateInfo,
}

vk_builder!(vk::PipelineTessellationStateCreateInfo, Builder);

impl Default for Builder {
  fn default() -> Builder {
    Builder {
      info: vk::PipelineTessellationStateCreateInfo {
        sType: vk::STRUCTURE_TYPE_PIPELINE_TESSELLATION_STATE_CREATE_INFO,
        pNext: std::ptr::null(),
        flags: 0,
        patchControlPoints: 0,
      },
    }
  }
}

impl Builder {
  pub fn patch_control_points(mut self, points: u32) -> Self {
    self.info.patchControlPoints = points;
    self
  }
}
