use vk;

/// Builder for a dynamic state
///
/// Default initialized with
///
/// - count: `0`
/// - states: none
pub struct Builder {
  pub states: Vec<vk::DynamicState>,
  pub info: vk::PipelineDynamicStateCreateInfo,
}

impl Builder {
  pub fn raw(info: vk::PipelineDynamicStateCreateInfo) -> Self {
    Self {
      states: Default::default(),
      info,
    }
  }

  pub fn push_state(mut self, state: vk::DynamicState) -> Self {
    self.states.push(state);
    self.update()
  }

  fn update(mut self) -> Self {
    self.info.dynamicStateCount = self.states.len() as u32;
    self.info.pDynamicStates = self.states.as_ptr();
    self
  }
}

impl Default for Builder {
  fn default() -> Builder {
    Builder {
      states: Default::default(),
      info: vk::PipelineDynamicStateCreateInfo {
        sType: vk::STRUCTURE_TYPE_PIPELINE_DYNAMIC_STATE_CREATE_INFO,
        pNext: std::ptr::null(),
        flags: 0,
        dynamicStateCount: 0,
        pDynamicStates: std::ptr::null(),
      },
    }
  }
}
