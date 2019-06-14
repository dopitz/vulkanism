use vk;

/// Builder for a dynamic state
///
/// Default initialized with
///
/// - count: `0`
/// - states: none
#[derive(Debug)]
pub struct Builder {
  states: Vec<vk::DynamicState>,
  info: vk::PipelineDynamicStateCreateInfo,
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
    self.info.dynamicStateCount = self.states.len() as u32;
    self.info.pDynamicStates = self.states.as_ptr();
    self
  }

  pub fn is_empty(&self) -> bool {
    self.states.is_empty()
  }
}
