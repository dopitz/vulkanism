use super::StreamPush;
use crate::cmd::CmdBuffer;
use vk;

/// Begins a render pass
#[derive(Clone, Copy)]
pub struct RenderpassBegin {
  pub info: vk::RenderPassBeginInfo,
  pub contents: vk::SubpassContents,
}

impl RenderpassBegin {
  pub fn new(pass: vk::RenderPass, framebuffer: vk::Framebuffer) -> RenderpassBegin {
    Self {
      info: vk::RenderPassBeginInfo {
        sType: vk::STRUCTURE_TYPE_RENDER_PASS_BEGIN_INFO,
        pNext: std::ptr::null(),
        renderPass: pass,
        framebuffer: framebuffer,
        renderArea: vk::Rect2D {
          offset: vk::Offset2D { x: 0, y: 0 },
          extent: vk::Extent2D { width: 0, height: 0 },
        },
        clearValueCount: 0,
        pClearValues: std::ptr::null(),
      },
      contents: vk::SUBPASS_CONTENTS_INLINE,
    }
  }

  pub fn contents(mut self, contents: vk::SubpassContents) -> Self {
    self.contents = contents;
    self
  }

  pub fn offset(mut self, offset: vk::Offset2D) -> Self {
    self.info.renderArea.offset = offset;
    self
  }
  pub fn extent(mut self, extent: vk::Extent2D) -> Self {
    self.info.renderArea.extent = extent;
    self
  }
  pub fn area(mut self, area: vk::Rect2D) -> Self {
    self.info.renderArea = area;
    self
  }

  pub fn clear(mut self, clear: &[vk::ClearValue]) -> Self {
    self.info.clearValueCount = clear.len() as u32;
    self.info.pClearValues = clear.as_ptr();
    self
  }
}

impl StreamPush for RenderpassBegin {
  fn enqueue(&self, cs: CmdBuffer) -> CmdBuffer {
    vk::CmdBeginRenderPass(cs.buffer, &self.info, self.contents);
    cs
  }
}

/// Ends a render pass
#[derive(Clone, Copy)]
pub struct RenderpassEnd {}

impl StreamPush for RenderpassEnd {
  fn enqueue(&self, cs: CmdBuffer) -> CmdBuffer {
    vk::CmdEndRenderPass(cs.buffer);
    cs
  }
}
