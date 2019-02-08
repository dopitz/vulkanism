use vk;

/// Builder for a raster state
///
/// Default initialized with
///
/// - depth clamp: dispabled
/// - rasterizer discard: disabled
/// - polygon mode: `vk::POLYGON_MODE_FILL`
/// - line width: `1`
/// - cull mode: `vk::CULL_MODE_BACK_BIT`
/// - front face: `vk::FRONT_FACE_COUNTER_CLOCKWISE`
/// - depth bias: disabled
/// - depth bias const factor: `0`
/// - depth bias clamp: `0`
/// - depth bias solpe factor: `0`
pub struct Builder {
  info: vk::PipelineRasterizationStateCreateInfo,
}

impl Builder {
  pub fn depth_clamp_enable(&mut self, enable: vk::Bool32) -> &mut Self {
    self.info.depthClampEnable = enable;
    self
  }
  pub fn discard_enable(&mut self, enable: vk::Bool32) -> &mut Self {
    self.info.rasterizerDiscardEnable = enable;
    self
  }
  pub fn polygon_mode(&mut self, mode: vk::PolygonMode) -> &mut Self {
    self.info.polygonMode = mode;
    self
  }
  pub fn line_width(&mut self, width: f32) -> &mut Self {
    self.info.lineWidth = width;
    self
  }
  pub fn cull_mode(&mut self, mode: vk::CullModeFlags) -> &mut Self {
    self.info.cullMode = mode;
    self
  }
  pub fn front_face(&mut self, front_face: vk::FrontFace) -> &mut Self {
    self.info.frontFace = front_face;
    self
  }
  pub fn depth_bias_enable(&mut self, enable: vk::Bool32) -> &mut Self {
    self.info.depthBiasEnable = enable;
    self
  }
  pub fn depth_bias_constantfactor(&mut self, f: f32) -> &mut Self {
    self.info.depthBiasConstantFactor = f;
    self
  }
  pub fn depth_bias_clamp(&mut self, c: f32) -> &mut Self {
    self.info.depthBiasClamp = c;
    self
  }
  pub fn depth_bias_slopefactor(&mut self, f: f32) -> &mut Self {
    self.info.depthBiasSlopeFactor = f;
    self
  }

  pub fn get(&self) -> &vk::PipelineRasterizationStateCreateInfo {
    &self.info
  }
}

impl Default for Builder {
  fn default() -> Builder {
    Builder {
      info: vk::PipelineRasterizationStateCreateInfo {
        sType: vk::STRUCTURE_TYPE_PIPELINE_RASTERIZATION_STATE_CREATE_INFO,
        pNext: std::ptr::null(),
        flags: 0,
        depthClampEnable: vk::FALSE,
        rasterizerDiscardEnable: vk::FALSE,
        polygonMode: vk::POLYGON_MODE_FILL,
        lineWidth: 1.0f32,
        cullMode: vk::CULL_MODE_BACK_BIT,
        frontFace: vk::FRONT_FACE_COUNTER_CLOCKWISE,
        depthBiasEnable: vk::FALSE,
        depthBiasConstantFactor: 0.0f32,
        depthBiasClamp: 0.0f32,
        depthBiasSlopeFactor: 0.0f32,
      },
    }
  }
}
