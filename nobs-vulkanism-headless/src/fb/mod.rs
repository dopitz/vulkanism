//! Manages vulkan framebuffers and renderpasses
//!
//! This module implements builder patterns for vulkan framebuffers and renderpasses.
//! Using [Renderpass](renderpass/struct.Renderpass.html) has the advantage of creating a framebuffer from it,
//! without having to manually create textures for all attachments.
//! [Framebuffer](framebuffer/struct.Framebuffer.html) implements begin and end [commands](../cmd/index.html).

pub mod framebuffer;
pub mod renderpass;

pub use framebuffer::Framebuffer;
pub use renderpass::Renderpass;

#[derive(Debug)]
pub enum Error {
  MissingSubpass(usize),
  NoDepthAttachmentConfigured,
  CreateRenderPass(vk::Error),
}

/// Returns a [builder](renderpass/struct.AttachmentBuilder.html) for an attachment for use in [Renderpass](renderpass/struct.Renderpass.html)
pub fn new_attachment(format: vk::Format) -> renderpass::AttachmentBuilder {
  renderpass::AttachmentBuilder::with_format(format)
}

/// Returns a [builder](renderpass/struct.SubpassBuilder.html) for a subpass for use in [Renderpass](renderpass/struct.Renderpass.html)
pub fn new_subpass(bindpoint: vk::PipelineBindPoint) -> renderpass::SubpassBuilder {
  renderpass::SubpassBuilder::with_bindpoint(bindpoint)
}

/// Returns a [builder](renderpass/struct.DependencyBuilder.html) for a subpass depencency for use in [Renderpass](renderpass/struct.Renderpass.html)
pub fn new_dependency() -> renderpass::DependencyBuilder {
  renderpass::DependencyBuilder::new()
}

/// Returns a [builder](renderpass/struct.Builder.html) for a [Renderpass](renderpass/struct.Renderpass.html)
pub fn new_pass(device: vk::Device) -> renderpass::Builder {
  renderpass::Builder::new(device)
}

/// Returns a [builder](framebuffer/struct.Builder.html) for a [Framebuffer](framebuffer/struct.Framebuffer.html)
pub fn new_framebuffer(device: vk::Device, pass: vk::RenderPass) -> framebuffer::Builder {
  framebuffer::Builder::new(device, pass)
}

/// Returns a [builder](framebuffer/struct.RenderpassFramebufferBuilder.html) for a [Framebuffer](framebuffer/struct.Framebuffer.html)
pub fn new_framebuffer_from_pass<'a, 'b>(
  pass: &'b Renderpass,
  alloc: &'a mut crate::mem::Allocator,
) -> framebuffer::RenderpassFramebufferBuilder<'a, 'b> {
  framebuffer::RenderpassFramebufferBuilder::new(pass, alloc)
}

/// Get a `vk::ClearValue` for colors initialized from 4 floats
pub fn clear_colorf32(c: [f32; 4]) -> vk::ClearValue {
  vk::ClearValue {
    color: vk::ClearColorValue { float32: c },
  }
}
/// Get a `vk::ClearValue` for colors initialized from 4 uints
pub fn clear_coloru32(c: [u32; 4]) -> vk::ClearValue {
  vk::ClearValue {
    color: vk::ClearColorValue { uint32: c },
  }
}
/// Get a `vk::ClearValue` for colors initialized from 4 ints
pub fn clear_colori32(c: [i32; 4]) -> vk::ClearValue {
  vk::ClearValue {
    color: vk::ClearColorValue { int32: c },
  }
}

/// Get a `vk::ClearValue` for depth
pub fn clear_depth(depth: f32) -> vk::ClearValue {
  vk::ClearValue {
    depthStencil: vk::ClearDepthStencilValue { depth, stencil: 0 },
  }
}
/// Get a `vk::ClearValue` for depth and stencil
pub fn clear_depth_stencil(depth: f32, stencil: u32) -> vk::ClearValue {
  vk::ClearValue {
    depthStencil: vk::ClearDepthStencilValue { depth, stencil },
  }
}

/// All supported depth texture formats
pub const DEPTH_FORMATS: &[vk::Format] = &[
  vk::FORMAT_D32_SFLOAT,
  vk::FORMAT_D32_SFLOAT_S8_UINT,
  vk::FORMAT_D24_UNORM_S8_UINT,
  vk::FORMAT_D16_UNORM,
  vk::FORMAT_X8_D24_UNORM_PACK32,
  vk::FORMAT_S8_UINT,
  vk::FORMAT_D16_UNORM_S8_UINT,
];

/// Select the best matching depth format for the specified physical device
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
