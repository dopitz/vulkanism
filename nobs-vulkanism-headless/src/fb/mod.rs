mod framebuffer;
mod renderpass;

pub use framebuffer::Framebuffer;
pub use renderpass::AttachmentBuilder;
pub use renderpass::Builder as RenderpassBuilder;
pub use renderpass::DependencyBuilder;
pub use renderpass::Renderpass;
pub use renderpass::SubpassBuilder;

#[derive(Debug)]
pub enum Error {
  MissingSubpass(usize),
  NoDepthAttachmentConfigured,
  CreateRenderPass(vk::Error),
}

pub fn new_attachment(format: vk::Format) -> renderpass::AttachmentBuilder {
  renderpass::AttachmentBuilder::with_format(format)
}

pub fn new_subpass(bindpoint: vk::PipelineBindPoint) -> renderpass::SubpassBuilder {
  renderpass::SubpassBuilder::with_bindpoint(bindpoint)
}

pub fn new_dependency() -> renderpass::DependencyBuilder {
  renderpass::DependencyBuilder::new()
}

pub fn new_pass(device: vk::Device) -> renderpass::Builder {
  renderpass::Builder::new(device)
}

pub fn new_framebuffer(device: vk::Device, pass: vk::RenderPass) -> framebuffer::Builder {
  framebuffer::Builder::new(device, pass)
}

pub fn new_framebuffer_from_pass<'a, 'b>(
  pass: &'b Renderpass,
  alloc: &'a mut crate::mem::Allocator,
) -> framebuffer::RenderpassFramebufferBuilder<'a, 'b> {
  framebuffer::RenderpassFramebufferBuilder::new(pass, alloc)
}

pub fn clear_colorf32(c: [f32; 4]) -> vk::ClearValue {
  vk::ClearValue {
    color: vk::ClearColorValue { float32: c },
  }
}
pub fn clear_coloru32(c: [u32; 4]) -> vk::ClearValue {
  vk::ClearValue {
    color: vk::ClearColorValue { uint32: c },
  }
}
pub fn clear_colori32(c: [i32; 4]) -> vk::ClearValue {
  vk::ClearValue {
    color: vk::ClearColorValue { int32: c },
  }
}

pub fn clear_depth(depth: f32) -> vk::ClearValue {
  vk::ClearValue {
    depthStencil: vk::ClearDepthStencilValue { depth, stencil: 0 },
  }
}
pub fn clear_depth_stencil(depth: f32, stencil: u32) -> vk::ClearValue {
  vk::ClearValue {
    depthStencil: vk::ClearDepthStencilValue { depth, stencil },
  }
}

pub const DEPTH_FORMATS: &[vk::Format] = &[
  vk::FORMAT_D32_SFLOAT,
  vk::FORMAT_D32_SFLOAT_S8_UINT,
  vk::FORMAT_D24_UNORM_S8_UINT,
  vk::FORMAT_D16_UNORM,
  vk::FORMAT_X8_D24_UNORM_PACK32,
  vk::FORMAT_S8_UINT,
  vk::FORMAT_D16_UNORM_S8_UINT,
];

pub fn select_depth_format(pdevice: vk::PhysicalDevice, formats: &[vk::Format]) -> Option<vk::Format> {
  formats
    .iter()
    .find(|f| {
      let mut props = unsafe { std::mem::uninitialized() };
      vk::GetPhysicalDeviceFormatProperties(pdevice, **f, &mut props);

      (props.optimalTilingFeatures & vk::FORMAT_FEATURE_DEPTH_STENCIL_ATTACHMENT_BIT) != 0
    })
    .cloned()
}
