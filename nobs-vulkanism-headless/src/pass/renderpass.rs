use crate::pass::Error;
use std::collections::HashMap;
use vk;

/// Managed vulkan rendepass
///
/// Tracks the lifetime and automatically destroys the vulkan handle when dropped.
///
/// The main advantage of `Renderpass` is, that with it we can conveniently create framebuffers from it with the [specialized builder](struct.RenderpassFramebufferBuilder.thml).
/// This way textures for every attachment are created automatically if nothing else is specified.
pub struct Renderpass {
  pub device: vk::Device,
  pub pass: vk::RenderPass,
  pub attachments: Vec<vk::AttachmentDescription>,
}

impl Drop for Renderpass {
  fn drop(&mut self) {
    vk::DestroyRenderPass(self.device, self.pass, std::ptr::null());
  }
}

impl Renderpass {
  pub fn build(device: vk::Device) -> Builder {
    Builder::new(device)
  }
}

/// Builder pattern for vk::AttachmentDescription
#[derive(Clone, Copy)]
pub struct AttachmentBuilder {
  desc: vk::AttachmentDescription,
}

vk_builder_into!(vk::AttachmentDescription, AttachmentBuilder, desc);

impl Default for AttachmentBuilder {
  fn default() -> Self {
    Self {
      desc: vk::AttachmentDescription {
        flags: 0,
        format: vk::FORMAT_UNDEFINED,
        samples: vk::SAMPLE_COUNT_1_BIT,
        loadOp: vk::ATTACHMENT_LOAD_OP_CLEAR,
        storeOp: vk::ATTACHMENT_STORE_OP_STORE,
        stencilLoadOp: vk::ATTACHMENT_LOAD_OP_DONT_CARE,
        stencilStoreOp: vk::ATTACHMENT_STORE_OP_DONT_CARE,
        initialLayout: vk::IMAGE_LAYOUT_UNDEFINED,
        finalLayout: vk::IMAGE_LAYOUT_PRESENT_SRC_KHR,
      },
    }
  }
}

impl AttachmentBuilder {
  pub fn format(mut self, format: vk::Format) -> Self {
    self.desc.format = format;
    self
  }
  pub fn samples(mut self, samples: vk::SampleCountFlags) -> Self {
    self.desc.samples = samples;
    self
  }
  pub fn load_op(mut self, op: vk::AttachmentLoadOp) -> Self {
    self.desc.loadOp = op;
    self
  }
  pub fn store_op(mut self, op: vk::AttachmentStoreOp) -> Self {
    self.desc.storeOp = op;
    self
  }
  pub fn stencil_load_op(mut self, op: vk::AttachmentLoadOp) -> Self {
    self.desc.stencilLoadOp = op;
    self
  }
  pub fn stencil_store_op(mut self, op: vk::AttachmentStoreOp) -> Self {
    self.desc.stencilStoreOp = op;
    self
  }
  pub fn initial_layout(mut self, layout: vk::ImageLayout) -> Self {
    self.desc.initialLayout = layout;
    self
  }
  pub fn final_layout(mut self, layout: vk::ImageLayout) -> Self {
    self.desc.finalLayout = layout;
    self
  }
}

/// Builder pattern for vk::SubpassDescription
pub struct SubpassBuilder {
  color: Vec<vk::AttachmentReference>,
  input: Vec<vk::AttachmentReference>,
  resolve: Vec<vk::AttachmentReference>,
  depth: Box<vk::AttachmentReference>,
  preserve: Vec<u32>,

  desc: vk::SubpassDescription,
}

vk_builder!(vk::SubpassDescription, SubpassBuilder, desc);

impl Default for SubpassBuilder {
  fn default() -> Self {
    Self {
      color: Default::default(),
      input: Default::default(),
      resolve: Default::default(),
      depth: Box::new(vk::AttachmentReference {
        attachment: vk::ATTACHMENT_UNUSED,
        layout: vk::IMAGE_LAYOUT_UNDEFINED,
      }),
      preserve: Default::default(),
      desc: vk::SubpassDescription {
        flags: 0,
        pipelineBindPoint: 0,
        colorAttachmentCount: 0,
        pColorAttachments: std::ptr::null(),
        inputAttachmentCount: 0,
        pInputAttachments: std::ptr::null(),
        pResolveAttachments: std::ptr::null(),
        pDepthStencilAttachment: std::ptr::null(),
        preserveAttachmentCount: 0,
        pPreserveAttachments: std::ptr::null(),
      },
    }
  }
}

impl SubpassBuilder {
  pub fn bindpoint(mut self, bindpoint: vk::PipelineBindPoint) -> Self {
    self.desc.pipelineBindPoint = bindpoint;
    self
  }

  pub fn color_layout(mut self, attachment: u32, layout: vk::ImageLayout) -> Self {
    self.color.push(vk::AttachmentReference { attachment, layout });
    self.desc.colorAttachmentCount = self.color.len() as u32;
    self.desc.pColorAttachments = self.color.as_ptr();
    self
  }
  pub fn color(self, attachment: u32) -> Self {
    self.color_layout(attachment, vk::IMAGE_LAYOUT_COLOR_ATTACHMENT_OPTIMAL)
  }

  pub fn input_layout(mut self, attachment: u32, layout: vk::ImageLayout) -> Self {
    self.input.push(vk::AttachmentReference { attachment, layout });
    self.desc.inputAttachmentCount = self.input.len() as u32;
    self.desc.pInputAttachments = self.input.as_ptr();
    self
  }
  pub fn input(self, attachment: u32) -> Self {
    self.input_layout(attachment, vk::IMAGE_LAYOUT_COLOR_ATTACHMENT_OPTIMAL)
  }

  pub fn resolve(mut self, attachment: u32, layout: vk::ImageLayout) -> Self {
    self.resolve.push(vk::AttachmentReference { attachment, layout });
    self.desc.pResolveAttachments = self.resolve.as_ptr();
    self
  }

  pub fn depth_layout(mut self, attachment: u32, layout: vk::ImageLayout) -> Self {
    self.depth = Box::new(vk::AttachmentReference { attachment, layout });
    self.desc.pDepthStencilAttachment = self.depth.as_ref();
    self
  }
  pub fn depth(self, attachment: u32) -> Self {
    self.depth_layout(attachment, vk::IMAGE_LAYOUT_DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
  }

  pub fn preserve(mut self, attachment: u32) -> Self {
    self.preserve.push(attachment);
    self.desc.preserveAttachmentCount = self.preserve.len() as u32;
    self.desc.pPreserveAttachments = self.preserve.as_ptr();
    self
  }
}

/// Builder pattern for vk::SubpassDependency
#[derive(Clone, Copy)]
pub struct DependencyBuilder {
  desc: vk::SubpassDependency,
}

vk_builder_into!(vk::SubpassDependency, DependencyBuilder, desc);

impl Default for DependencyBuilder {
  fn default() -> Self {
    Self {
      desc: vk::SubpassDependency {
        dependencyFlags: 0,
        srcSubpass: 0,
        dstSubpass: 0,
        srcStageMask: 0,
        srcAccessMask: 0,
        dstStageMask: 0,
        dstAccessMask: 0,
      },
    }
  }
}

impl DependencyBuilder {
  pub fn external(self, dstpass: u32) -> Self {
    self
      .src(vk::SUBPASS_EXTERNAL, vk::PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT, 0)
      .dst(
        dstpass,
        vk::PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT,
        vk::ACCESS_COLOR_ATTACHMENT_READ_BIT | vk::ACCESS_COLOR_ATTACHMENT_WRITE_BIT,
      )
  }
  pub fn src(mut self, pass: u32, stages: vk::PipelineStageFlags, access: vk::AccessFlags) -> Self {
    self.desc.srcSubpass = pass;
    self.desc.srcStageMask = stages;
    self.desc.srcAccessMask = access;
    self
  }
  pub fn dst(mut self, pass: u32, stages: vk::PipelineStageFlags, access: vk::AccessFlags) -> Self {
    self.desc.dstSubpass = pass;
    self.desc.dstStageMask = stages;
    self.desc.dstAccessMask = access;
    self
  }
}

/// Builder for [Renderpass](struct.Renderpass.html)
pub struct Builder {
  pub device: vk::Device,

  pub attachments: HashMap<u32, vk::AttachmentDescription>,
  pub depth: Option<u32>,

  pub subpasses: HashMap<u32, SubpassBuilder>,
  pub dependencies: Vec<vk::SubpassDependency>,
}

impl Builder {
  /// Build a renderpass for the specified device
  pub fn new(device: vk::Device) -> Self {
    Self {
      device,
      attachments: Default::default(),
      depth: None,
      subpasses: Default::default(),
      dependencies: Default::default(),
    }
  }

  /// Adds an attachment at position `index`
  pub fn attachment(&mut self, index: u32, builder: AttachmentBuilder) -> &mut Self {
    let desc = self.attachments.entry(index).or_insert_with(|| builder.into());
    if super::Framebuffer::enumerate_depth_formats()
      .iter()
      .find(|f| **f == desc.format)
      .is_some()
    {
      self.depth = Some(index);
    }
    self
  }

  /// Adds the subpass at position `index`
  pub fn subpass(&mut self, index: u32, builder: SubpassBuilder) -> &mut Self {
    self.subpasses.entry(index).or_insert_with(|| builder);
    self
  }

  /// Adds a subpass dependency
  pub fn dependency(&mut self, builder: DependencyBuilder) -> &mut Self {
    self.dependencies.push(builder.into());
    self
  }

  /// Create the Renderpass
  pub fn create(&self) -> Result<Renderpass, Error> {
    for i in 0..self.subpasses.len() {
      match self.subpasses.get(&(i as u32)) {
        None => Err(Error::MissingSubpass(i))?,
        Some(b) => {
          if !(b.depth.attachment == vk::ATTACHMENT_UNUSED || self.depth.is_some()) {
            Err(Error::NoDepthAttachmentConfigured)?;
          }
        }
      }
    }

    let mut subpasses = Vec::with_capacity(self.subpasses.len());
    for i in 0..self.subpasses.len() {
      subpasses.push(*self.subpasses[&(i as u32)].as_ref());
    }
    let mut attachments = Vec::with_capacity(self.attachments.len());
    for i in 0..self.attachments.len() {
      attachments.push(self.attachments[&(i as u32)]);
    }

    let info = vk::RenderPassCreateInfo {
      sType: vk::STRUCTURE_TYPE_RENDER_PASS_CREATE_INFO,
      pNext: std::ptr::null(),
      flags: 0,
      attachmentCount: attachments.len() as u32,
      pAttachments: attachments.as_ptr(),
      subpassCount: subpasses.len() as u32,
      pSubpasses: subpasses.as_ptr(),
      dependencyCount: self.dependencies.len() as u32,
      pDependencies: self.dependencies.as_ptr(),
    };

    let mut pass = vk::NULL_HANDLE;
    vk_check!(vk::CreateRenderPass(self.device, &info, std::ptr::null(), &mut pass)).map_err(|e| Error::CreateRenderPass(e))?;
    Ok(Renderpass {
      device: self.device,
      pass,
      attachments: attachments.clone(),
    })
  }
}
