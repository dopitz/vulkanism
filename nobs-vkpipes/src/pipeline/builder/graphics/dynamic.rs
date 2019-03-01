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

vk_builder!(vk::PipelineDynamicStateCreateInfo, Builder);

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

impl Builder {
  pub fn push_state(mut self, state: vk::DynamicState) -> Self {
    self.states.push(state);
    self
  }

  pub fn get(&self) -> vk::PipelineDynamicStateCreateInfo {
    let mut info = self.info;
    if info.pDynamicStates.is_null() && !self.states.is_empty() {
      info.dynamicStateCount = self.states.len() as u32;
      info.pDynamicStates = self.states.as_ptr();
    }
    info
  }
}
