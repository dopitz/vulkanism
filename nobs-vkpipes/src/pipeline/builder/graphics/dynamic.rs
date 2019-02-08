use vk;

/// Builder for a dynamic state
///
/// Default initialized with
///
/// - count: 0
/// - states: none
pub struct Builder {
  states: Vec<vk::DynamicState>,
  info: vk::PipelineDynamicStateCreateInfo,
}

impl Builder {
  pub fn push_state(&mut self, state: vk::DynamicState) -> &mut Self {
    self.states.push(state);
    self.update()
  }

  fn update(&mut self) -> &mut Self {
    self.info.dynamicStateCount = self.states.len() as u32;
    self.info.pDynamicStates = self.states.as_ptr();
    self
  }

  pub fn is_empty(&self) -> bool {
    self.states.is_empty()
  }

  pub fn get(&self) -> &vk::PipelineDynamicStateCreateInfo {
    &self.info
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
