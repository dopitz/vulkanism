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

impl Builder {
  pub fn push_binding(&mut self, binding: vk::VertexInputBindingDescription) -> &mut Self {
    self.bindings.push(binding);
    self.update()
  }
  pub fn push_bindings(&mut self, bindings: &mut [vk::VertexInputBindingDescription]) -> &mut Self {
    bindings.iter().for_each(|b| self.bindings.push(*b));
    self.update()
  }

  pub fn push_attribute(&mut self, attrib: vk::VertexInputAttributeDescription) -> &mut Self {
    self.attributes.push(attrib);
    self.update()
  }
  pub fn push_attributes(&mut self, attribs: &mut [vk::VertexInputAttributeDescription]) -> &mut Self {
    attribs.iter().for_each(|a| self.attributes.push(*a));
    self.update()
  }

  fn update(&mut self) -> &mut Self {
    self.info.vertexBindingDescriptionCount = self.bindings.len() as u32;
    self.info.pVertexBindingDescriptions = self.bindings.as_ptr();
    self.info.vertexAttributeDescriptionCount = self.attributes.len() as u32;
    self.info.pVertexAttributeDescriptions = self.attributes.as_ptr();
    self
  }

  pub fn get(&self) -> &vk::PipelineVertexInputStateCreateInfo {
    &self.info
  }
}

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
