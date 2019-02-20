use std::collections::HashMap;

use vk;

use crate::fb::Error;

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

#[derive(Clone, Copy)]
pub struct AttachmentBuilder {
  pub desc: vk::AttachmentDescription,
}

impl AttachmentBuilder {
  pub fn with_format(format: vk::Format) -> Self {
    Self {
      desc: vk::AttachmentDescription {
        flags: 0,
        format: format,
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

  pub fn with_rawdesc(desc: vk::AttachmentDescription) -> Self {
    Self { desc }
  }

  pub fn samples(&mut self, samples: vk::SampleCountFlags) -> &mut Self {
    self.desc.samples = samples;
    self
  }
  pub fn load_op(&mut self, op: vk::AttachmentLoadOp) -> &mut Self {
    self.desc.loadOp = op;
    self
  }
  pub fn store_op(&mut self, op: vk::AttachmentStoreOp) -> &mut Self {
    self.desc.storeOp = op;
    self
  }
  pub fn stencil_load_op(&mut self, op: vk::AttachmentLoadOp) -> &mut Self {
    self.desc.stencilLoadOp = op;
    self
  }
  pub fn stencil_store_op(&mut self, op: vk::AttachmentStoreOp) -> &mut Self {
    self.desc.stencilStoreOp = op;
    self
  }
  pub fn initial_layout(&mut self, layout: vk::ImageLayout) -> &mut Self {
    self.desc.initialLayout = layout;
    self
  }
  pub fn final_layout(&mut self, layout: vk::ImageLayout) -> &mut Self {
    self.desc.finalLayout = layout;
    self
  }

  pub fn get_desc(&self) -> vk::AttachmentDescription {
    self.desc
  }
}

#[derive(Clone)]
pub struct SubpassBuilder {
  pub input: Vec<vk::AttachmentReference>,
  pub color: Vec<vk::AttachmentReference>,
  pub resolve: Vec<vk::AttachmentReference>,
  pub depth: vk::AttachmentReference,
  pub preserve: Vec<u32>,

  pub desc: vk::SubpassDescription,
}

impl SubpassBuilder {
  pub fn with_bindpoint(bindpoint: vk::PipelineBindPoint) -> Self {
    Self {
      input: Default::default(),
      color: Default::default(),
      resolve: Default::default(),
      depth: vk::AttachmentReference {
        attachment: vk::ATTACHMENT_UNUSED,
        layout: vk::IMAGE_LAYOUT_UNDEFINED,
      },
      preserve: Default::default(),
      desc: vk::SubpassDescription {
        flags: 0,
        pipelineBindPoint: bindpoint,
        inputAttachmentCount: 0,
        pInputAttachments: std::ptr::null(),
        colorAttachmentCount: 0,
        pColorAttachments: std::ptr::null(),
        pResolveAttachments: std::ptr::null(),
        pDepthStencilAttachment: std::ptr::null(),
        preserveAttachmentCount: 0,
        pPreserveAttachments: std::ptr::null(),
      },
    }
  }

  pub fn with_rawdesc(desc: vk::SubpassDescription) -> Self {
    Self {
      input: Default::default(),
      color: Default::default(),
      resolve: Default::default(),
      depth: vk::AttachmentReference {
        attachment: vk::ATTACHMENT_UNUSED,
        layout: vk::IMAGE_LAYOUT_UNDEFINED,
      },
      preserve: Default::default(),
      desc,
    }
  }

  pub fn color_layout(&mut self, attachment: u32, layout: vk::ImageLayout) -> &mut Self {
    self.color.push(vk::AttachmentReference { attachment, layout });
    self.desc.colorAttachmentCount = self.color.len() as u32;
    self.desc.pColorAttachments = self.color.as_ptr();
    self
  }
  pub fn color(&mut self, attachment: u32) -> &mut Self {
    self.color_layout(attachment, vk::IMAGE_LAYOUT_COLOR_ATTACHMENT_OPTIMAL)
  }

  pub fn input_layout(&mut self, attachment: u32, layout: vk::ImageLayout) -> &mut Self {
    self.input.push(vk::AttachmentReference { attachment, layout });
    self.desc.inputAttachmentCount = self.input.len() as u32;
    self.desc.pInputAttachments = self.input.as_ptr();
    self
  }
  pub fn input(&mut self, attachment: u32) -> &mut Self {
    self.input_layout(attachment, vk::IMAGE_LAYOUT_COLOR_ATTACHMENT_OPTIMAL)
  }

  pub fn resolve(&mut self, attachment: u32, layout: vk::ImageLayout) -> &mut Self {
    self.resolve.push(vk::AttachmentReference { attachment, layout });
    self.desc.pResolveAttachments = self.resolve.as_ptr();
    self
  }

  pub fn depth_layout(&mut self, attachment: u32, layout: vk::ImageLayout) -> &mut Self {
    self.depth = vk::AttachmentReference { attachment, layout };
    self.desc.pDepthStencilAttachment = &self.depth;
    self
  }
  pub fn depth(&mut self, attachment: u32) -> &mut Self {
    self.depth_layout(attachment, vk::IMAGE_LAYOUT_DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
  }

  pub fn preserve(&mut self, attachment: u32) -> &mut Self {
    self.preserve.push(attachment);
    self.desc.pPreserveAttachments = self.preserve.as_ptr();
    self
  }

  pub fn get_desc(&self) -> vk::SubpassDescription {
    self.desc
  }
}

#[derive(Clone, Copy)]
pub struct DependencyBuilder {
  pub desc: vk::SubpassDependency,
}

impl DependencyBuilder {
  pub fn new() -> Self {
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

  pub fn with_rawdesc(desc: vk::SubpassDependency) -> Self {
    Self { desc }
  }

  pub fn external(&mut self, dstpass: u32) -> &mut Self {
    self
      .src(vk::SUBPASS_EXTERNAL, vk::PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT, 0)
      .dst(
        dstpass,
        vk::PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT,
        vk::ACCESS_COLOR_ATTACHMENT_READ_BIT | vk::ACCESS_COLOR_ATTACHMENT_WRITE_BIT,
      )
  }

  pub fn src(&mut self, pass: u32, stages: vk::PipelineStageFlags, access: vk::AccessFlags) -> &mut Self {
    self.desc.srcSubpass = pass;
    self.desc.srcStageMask = stages;
    self.desc.srcAccessMask = access;
    self
  }

  pub fn dst(&mut self, pass: u32, stages: vk::PipelineStageFlags, access: vk::AccessFlags) -> &mut Self {
    self.desc.dstSubpass = pass;
    self.desc.dstStageMask = stages;
    self.desc.dstAccessMask = access;
    self
  }

  pub fn get_desc(&self) -> vk::SubpassDependency {
    self.desc
  }
}

pub struct Builder {
  pub device: vk::Device,

  pub attachments: HashMap<u32, vk::AttachmentDescription>,
  pub depth: Option<u32>,

  pub subpasses: HashMap<u32, SubpassBuilder>,
  pub dependencies: Vec<vk::SubpassDependency>,
}

impl Builder {
  pub fn new(device: vk::Device) -> Self {
    Self {
      device,
      attachments: Default::default(),
      depth: None,
      subpasses: Default::default(),
      dependencies: Default::default(),
    }
  }

  pub fn attachment(&mut self, index: u32, builder: AttachmentBuilder) -> &mut Self {
    let desc = self.attachments.entry(index).or_insert_with(|| builder.get_desc());
    if crate::fb::DEPTH_FORMATS.iter().find(|f| **f == desc.format).is_some() {
      self.depth = Some(index);
    }
    self
  }

  pub fn subpass(&mut self, index: u32, builder: SubpassBuilder) -> &mut Self {
    self.subpasses.entry(index).or_insert_with(|| builder.clone());
    self
  }

  pub fn dependency(&mut self, builder: DependencyBuilder) -> &mut Self {
    self.dependencies.push(builder.get_desc());
    self
  }

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
      subpasses.push(self.subpasses[&(i as u32)].get_desc());
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
