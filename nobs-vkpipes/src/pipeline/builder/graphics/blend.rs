use vk;

/// Builder for a blend state
///
/// Default initialized with
///
/// - blend: disabled
/// - color write mask: `vk::COLOR_COMPONENT_R_BIT | vk::COLOR_COMPONENT_G_BIT | vk::COLOR_COMPONENT_B_BIT | vk::COLOR_COMPONENT_A_BIT`,
/// - src color blend factor: `vk::BLEND_FACTER_ONE`
/// - dst color blend factor: `vk::BLEND_FACTER_ZERO`
/// - color blend op: `vk::BLEND_OP_ADD`
/// - src alpha blend factor: `vk::BLEND_FACTER_ONE`
/// - dst alpha blend factor: `vk::BLEND_FACTER_ZERO`
/// - alpha blend op: `vk::BLEND_OP_ADD`
pub struct Builder {
  pub attachments: Vec<vk::PipelineColorBlendAttachmentState>,
  pub info: vk::PipelineColorBlendStateCreateInfo,
}

vk_builder!(vk::PipelineColorBlendStateCreateInfo, Builder);

impl Default for Builder {
  fn default() -> Builder {
    Builder {
      attachments: Default::default(),
      info: vk::PipelineColorBlendStateCreateInfo {
        sType: vk::STRUCTURE_TYPE_PIPELINE_COLOR_BLEND_STATE_CREATE_INFO,
        pNext: std::ptr::null(),
        flags: 0,
        logicOpEnable: vk::FALSE,
        logicOp: vk::LOGIC_OP_COPY,
        blendConstants: [0.0f32, 0.0f32, 0.0f32, 0.0f32],
        attachmentCount: 0,
        pAttachments: std::ptr::null(),
      },
    }
  }
}

impl Builder {
  pub fn push_attachment(mut self, b: AttachmentBuilder) -> Self {
    self.attachments.push(b.info);
    self
  }

  pub fn logic_op_enable(mut self, enable: vk::Bool32) -> Self {
    self.info.logicOpEnable = enable;
    self
  }
  pub fn logic_op(mut self, op: vk::LogicOp) -> Self {
    self.info.logicOp = op;
    self
  }
  pub fn blend_constants(mut self, consts: [f32; 4]) -> Self {
    self.info.blendConstants = consts;
    self
  }

  pub fn get(&self) -> vk::PipelineColorBlendStateCreateInfo {
    let mut info = self.info;
    if info.pAttachments.is_null() && !self.attachments.is_empty() {
      info.attachmentCount = self.attachments.len() as u32;
      info.pAttachments = self.attachments.as_ptr();
    }
    info
  }
}

pub struct AttachmentBuilder {
  pub info: vk::PipelineColorBlendAttachmentState,
}

vk_builder!(vk::PipelineColorBlendAttachmentState, AttachmentBuilder);

impl Default for AttachmentBuilder {
  fn default() -> AttachmentBuilder {
    AttachmentBuilder {
      info: vk::PipelineColorBlendAttachmentState {
        blendEnable: vk::FALSE,
        colorWriteMask: vk::COLOR_COMPONENT_R_BIT | vk::COLOR_COMPONENT_G_BIT | vk::COLOR_COMPONENT_B_BIT | vk::COLOR_COMPONENT_A_BIT,
        srcColorBlendFactor: vk::BLEND_FACTOR_ONE,
        dstColorBlendFactor: vk::BLEND_FACTOR_ZERO,
        colorBlendOp: vk::BLEND_OP_ADD,
        srcAlphaBlendFactor: vk::BLEND_FACTOR_ONE,
        dstAlphaBlendFactor: vk::BLEND_FACTOR_ZERO,
        alphaBlendOp: vk::BLEND_OP_ADD,
      },
    }
  }
}

impl AttachmentBuilder {
  pub fn enable(mut self, enable: vk::Bool32) -> Self {
    self.info.blendEnable = enable;
    self
  }
  pub fn color_write_mask(mut self, mask: vk::ColorComponentFlags) -> Self {
    self.info.colorWriteMask = mask;
    self
  }
  pub fn color(mut self, src: vk::BlendFactor, dst: vk::BlendFactor, op: vk::BlendOp) -> Self {
    self.info.srcColorBlendFactor = src;
    self.info.dstColorBlendFactor = dst;
    self.info.colorBlendOp = op;
    self
  }
  pub fn alpha(mut self, src: vk::BlendFactor, dst: vk::BlendFactor, op: vk::BlendOp) -> Self {
    self.info.srcAlphaBlendFactor = src;
    self.info.dstAlphaBlendFactor = dst;
    self.info.alphaBlendOp = op;
    self
  }
  pub fn color_and_alpha(self, src: vk::BlendFactor, dst: vk::BlendFactor, op: vk::BlendOp) -> Self {
    self.color(src, dst, op).alpha(src, dst, op)
  }
}
