//! Manages vulkan framebuffers and renderpasses
//!
//! This module implements builder patterns for vulkan framebuffers and renderpasses.
//! Using [Renderpass](renderpass/struct.Renderpass.html) has the advantage of creating a framebuffer from it,
//! without having to manually create textures for all attachments.
//! [Framebuffer](framebuffer/struct.Framebuffer.html) implements begin and end [commands](../cmd/index.html).
mod drawpass;
mod frame;
mod framebuffer;
mod renderpass;

pub use drawpass::DrawMeshRef;
pub use drawpass::DrawPass;
pub use frame::Frame;
pub use framebuffer::Framebuffer;
pub use renderpass::Renderpass;

#[derive(Debug)]
pub enum Error {
  MissingSubpass(usize),
  NoDepthAttachmentConfigured,
  CreateRenderPass(vk::Error),
}
