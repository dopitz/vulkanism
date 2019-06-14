use vk;

/// Builder for a vertex input state
///
/// Default initialized with
///
/// - no vertex bindings
/// - no vertex attributes
pub struct Builder {
  bindings: Vec<vk::VertexInputBindingDescription>,
  attributes: Vec<vk::VertexInputAttributeDescription>,
  info: vk::PipelineVertexInputStateCreateInfo,
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
  pub fn push_binding(mut self, binding: BindingBuilder) -> Self {
    self.bindings.push(binding.into());
    self.info.vertexBindingDescriptionCount = self.bindings.len() as u32;
    self.info.pVertexBindingDescriptions = self.bindings.as_ptr();
    self
  }

  pub fn push_attribute(mut self, attrib: AttributeBuilder) -> Self {
    self.attributes.push(attrib.into());
    self.info.vertexAttributeDescriptionCount = self.attributes.len() as u32;
    self.info.pVertexAttributeDescriptions = self.attributes.as_ptr();
    self
  }
}

/// Builder for a vertex input binding
pub struct BindingBuilder {
  binding: vk::VertexInputBindingDescription,
}

vk_builder_into!(vk::VertexInputBindingDescription, BindingBuilder, binding);

impl Default for BindingBuilder {
  fn default() -> Self {
    Self {
      binding: vk::VertexInputBindingDescription {
        binding: 0,
        inputRate: vk::VERTEX_INPUT_RATE_VERTEX,
        stride: 0,
      },
    }
  }
}

impl BindingBuilder {
  pub fn binding(mut self, binding: u32) -> Self {
    self.binding.binding = binding;
    self
  }

  pub fn input_rate(mut self, rate: vk::VertexInputRate) -> Self {
    self.binding.inputRate = rate;
    self
  }

  pub fn stride(mut self, stride: u32) -> Self {
    self.binding.stride = stride;
    self
  }
}

/// Builder for a vertex attribute
pub struct AttributeBuilder {
  attribute: vk::VertexInputAttributeDescription,
}

vk_builder_into!(vk::VertexInputAttributeDescription, AttributeBuilder, attribute);

impl Default for AttributeBuilder {
  fn default() -> Self {
    Self {
      attribute: vk::VertexInputAttributeDescription {
        binding: 0,
        format: vk::FORMAT_R32G32B32A32_SFLOAT,
        location: 0,
        offset: 0,
      },
    }
  }
}

impl AttributeBuilder {
  pub fn binding(mut self, binding: u32) -> Self {
    self.attribute.binding = binding;
    self
  }

  pub fn format(mut self, format: vk::Format) -> Self {
    self.attribute.format = format;
    self
  }

  pub fn location(mut self, location: u32) -> Self {
    self.attribute.location = location;
    self
  }

  pub fn offset(mut self, offset: u32) -> Self {
    self.attribute.offset = offset;
    self
  }
}
