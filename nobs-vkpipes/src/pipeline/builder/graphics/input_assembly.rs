use vk;

/// Builder for a vertex input state
///
/// Default initialized with
///
/// - primitive topology: `vk::PRIMITIVE_TOPOLOGY_TRIANGLE_LIST`
/// - primiteve restart: disabled
pub struct Builder {
  pub info: vk::PipelineInputAssemblyStateCreateInfo,
}

vk_builder!(vk::PipelineInputAssemblyStateCreateInfo, Builder);

impl Default for Builder {
  fn default() -> Builder {
    Builder {
      info: vk::PipelineInputAssemblyStateCreateInfo {
        sType: vk::STRUCTURE_TYPE_PIPELINE_INPUT_ASSEMBLY_STATE_CREATE_INFO,
        pNext: std::ptr::null(),
        flags: 0,
        topology: vk::PRIMITIVE_TOPOLOGY_TRIANGLE_LIST,
        primitiveRestartEnable: vk::FALSE,
      },
    }
  }
}

impl Builder {
  pub fn topology(mut self, topology: vk::PrimitiveTopology) -> Self {
    self.info.topology = topology;
    self
  }
  pub fn primitive_restart_enable(mut self, enable: vk::Bool32) -> Self {
    self.info.primitiveRestartEnable = enable;
    self
  }
}
