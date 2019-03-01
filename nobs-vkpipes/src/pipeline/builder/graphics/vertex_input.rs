use vk;

/// Builder for a vertex input state
///
/// Default initialized with
///
/// - no vertex bindings
/// - no vertex attributes
pub struct Builder {
  pub bindings: Vec<vk::VertexInputBindingDescription>,
  pub attributes: Vec<vk::VertexInputAttributeDescription>,
  pub info: vk::PipelineVertexInputStateCreateInfo,
}

vk_builder!(vk::PipelineVertexInputStateCreateInfo, Builder);

impl Default for Builder {
  fn default() -> Builder {
    Builder {
      bindings: Default::default(),
      attributes: Default::default(),
      info: vk::PipelineVertexInputStateCreateInfo {
        sType: vk::STRUCTURE_TYPE_PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO,
        pNext: std::ptr::null(),
        flags: 0,
        vertexBindingDescriptionCount: 0,
        pVertexBindingDescriptions: std::ptr::null(),
        vertexAttributeDescriptionCount: 0,
        pVertexAttributeDescriptions: std::ptr::null(),
      },
    }
  }
}

impl Builder {
  pub fn push_binding(mut self, binding: vk::VertexInputBindingDescription) -> Self {
    self.bindings.push(binding);
    self
  }
  pub fn push_bindings(mut self, bindings: &mut [vk::VertexInputBindingDescription]) -> Self {
    bindings.iter().for_each(|b| self.bindings.push(*b));
    self
  }

  pub fn push_attribute(mut self, attrib: vk::VertexInputAttributeDescription) -> Self {
    self.attributes.push(attrib);
    self
  }
  pub fn push_attributes(mut self, attribs: &mut [vk::VertexInputAttributeDescription]) -> Self {
    attribs.iter().for_each(|a| self.attributes.push(*a));
    self
  }

  pub fn get(&self) -> vk::PipelineVertexInputStateCreateInfo {
    let mut info = self.info;
    if info.pVertexBindingDescriptions.is_null() && !self.bindings.is_empty() {
      info.vertexBindingDescriptionCount = self.bindings.len() as u32;
      info.pVertexBindingDescriptions = self.bindings.as_ptr();
    }
    if info.pVertexAttributeDescriptions.is_null() && !self.attributes.is_empty() {
      info.vertexAttributeDescriptionCount = self.attributes.len() as u32;
      info.pVertexAttributeDescriptions = self.attributes.as_ptr();
    }
    info
  }
}
