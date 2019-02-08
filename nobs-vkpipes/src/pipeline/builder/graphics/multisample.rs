use vk;

/// Builder for a multisample state
///
/// Default initialized with
///
/// - sample shading: disabled
/// - rasterization samples: vk::SAMPLE_COUNT_1_BIT
/// - min sample shading: 1
/// - sample mask: null
/// - alpha to coverage: disabled
/// - alpha to one: disabled
pub struct Builder {
  info: vk::PipelineMultisampleStateCreateInfo,
}

impl Builder {
  pub fn sample_shading_enable(&mut self, enable: vk::Bool32) -> &mut Self {
    self.info.sampleShadingEnable = enable;
    self
  }
  pub fn rasterization_samples(&mut self, samples: vk::SampleCountFlagBits) -> &mut Self {
    self.info.rasterizationSamples = samples;
    self
  }
  pub fn min_sample_shading(&mut self, samples: f32) -> &mut Self {
    self.info.minSampleShading = samples;
    self
  }
  pub fn sample_mask(&mut self, mask: *const vk::SampleMask) -> &mut Self {
    self.info.pSampleMask = mask;
    self
  }
  pub fn alpha_to_coverage_enable(&mut self, enable: vk::Bool32) -> &mut Self {
    self.info.alphaToCoverageEnable = enable;
    self
  }
  pub fn alpha_to_one_enable(&mut self, enable: vk::Bool32) -> &mut Self {
    self.info.alphaToOneEnable = enable;
    self
  }

  pub fn get(&self) -> &vk::PipelineMultisampleStateCreateInfo {
    &self.info
  }
}

impl Default for Builder {
  fn default() -> Builder {
    Builder {
      info: vk::PipelineMultisampleStateCreateInfo {
        sType: vk::STRUCTURE_TYPE_PIPELINE_MULTISAMPLE_STATE_CREATE_INFO,
        pNext: std::ptr::null(),
        flags: 0,
        sampleShadingEnable: vk::FALSE,
        rasterizationSamples: vk::SAMPLE_COUNT_1_BIT,
        minSampleShading: 1.0f32,
        pSampleMask: std::ptr::null(),
        alphaToCoverageEnable: vk::FALSE,
        alphaToOneEnable: vk::FALSE,
      },
    }
  }
}
