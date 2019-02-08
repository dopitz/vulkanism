use vk;

/// Builder for a vertex input state
///
/// Default initialized with
///
/// - primitive topology: `vk::PRIMITIVE_TOPOLOGY_TRIANGLE_LIST`
/// - primiteve restart: disabled
pub struct Builder {
  info: vk::PipelineInputAssemblyStateCreateInfo,
}

impl Builder {
  pub fn topology(&mut self, topology: vk::PrimitiveTopology) -> &mut Self {
    self.info.topology = topology;
    self
  }
  pub fn primitive_restart_enable(&mut self, enable: vk::Bool32) -> &mut Self {
    self.info.primitiveRestartEnable = enable;
    self
  }

  pub fn get(&self) -> &vk::PipelineInputAssemblyStateCreateInfo {
    &self.info
  }
}

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
