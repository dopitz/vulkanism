use super::ImageBarrier;
use super::Stream;
use super::StreamPush;
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
  fn enqueue(&self, cs: Stream) -> Stream {
    vk::CmdBeginRenderPass(cs.buffer, &self.info, self.contents);
    cs
  }
}

/// Ends a render pass
pub struct RenderpassEnd {}

impl StreamPush for RenderpassEnd {
  fn enqueue(&self, cs: Stream) -> Stream {
    vk::CmdEndRenderPass(cs.buffer);
    cs
  }
}

/// Blits src to dst
#[derive(Clone, Copy)]
pub struct Blit {
  region: vk::ImageBlit,
  src: vk::Image,
  dst: vk::Image,
  filter: vk::Filter,
}

impl Blit {
  pub fn new() -> Blit {
    Self {
      region: vk::ImageBlit {
        srcSubresource: vk::ImageSubresourceLayers {
          aspectMask: vk::IMAGE_ASPECT_COLOR_BIT,
          mipLevel: 0,
          baseArrayLayer: 0,
          layerCount: 1,
        },
        srcOffsets: [vk::Offset3D { x: 0, y: 0, z: 0 }, vk::Offset3D { x: 0, y: 0, z: 0 }],
        dstSubresource: vk::ImageSubresourceLayers {
          aspectMask: vk::IMAGE_ASPECT_COLOR_BIT,
          mipLevel: 0,
          baseArrayLayer: 0,
          layerCount: 1,
        },
        dstOffsets: [vk::Offset3D { x: 0, y: 0, z: 0 }, vk::Offset3D { x: 0, y: 0, z: 0 }],
      },
      src: vk::NULL_HANDLE,
      dst: vk::NULL_HANDLE,
      filter: vk::FILTER_LINEAR,
    }
  }

  pub fn src(mut self, img: vk::Image) -> Self {
    self.src = img;
    self
  }

  pub fn src_subresource(mut self, subresource: vk::ImageSubresourceLayers) -> Self {
    self.region.srcSubresource = subresource;
    self
  }

  pub fn src_offset_begin(mut self, x: i32, y: i32, z: i32) -> Self {
    self.region.srcOffsets[0] = vk::Offset3D { x, y, z };
    self
  }

  pub fn src_offset_end(mut self, x: i32, y: i32, z: i32) -> Self {
    self.region.srcOffsets[1] = vk::Offset3D { x, y, z };
    self
  }

  pub fn dst(mut self, img: vk::Image) -> Self {
    self.dst = img;
    self
  }

  pub fn dst_subresource(mut self, subresource: vk::ImageSubresourceLayers) -> Self {
    self.region.dstSubresource = subresource;
    self
  }

  pub fn dst_offset_begin(mut self, x: i32, y: i32, z: i32) -> Self {
    self.region.dstOffsets[0] = vk::Offset3D { x, y, z };
    self
  }

  pub fn dst_offset_end(mut self, x: i32, y: i32, z: i32) -> Self {
    self.region.dstOffsets[1] = vk::Offset3D { x, y, z };
    self
  }

  pub fn filter(mut self, filter: vk::Filter) -> Self {
    self.filter = filter;
    self
  }
}

impl StreamPush for Blit {
  fn enqueue(&self, cs: Stream) -> Stream {
    let cs = cs
      .push(&ImageBarrier::to_transfer_src(self.src))
      .push(&ImageBarrier::to_transfer_dst(self.dst));

    vk::CmdBlitImage(
      cs.buffer,
      self.src,
      vk::IMAGE_LAYOUT_TRANSFER_SRC_OPTIMAL,
      self.dst,
      vk::IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL,
      1,
      &self.region,
      self.filter,
    );

    cs.push(&ImageBarrier::to_present(self.dst))
  }
}

/// Clear Color render target
#[derive(Clone, Copy)]
pub struct ClearColorImage {
  pub image: vk::Image,
  pub layout: vk::ImageLayout,
  pub clear: vk::ClearColorValue,
  pub subresource: vk::ImageSubresourceRange,
}

impl ClearColorImage {
  pub fn new(image: vk::Image) -> Self {
    Self {
      image,
      layout: vk::IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL,
      clear: vk::ClearColorValue { int32: [0, 0, 0, 0] },
      subresource: vk::ImageSubresourceRange {
        aspectMask: vk::IMAGE_ASPECT_COLOR_BIT,
        baseMipLevel: 0,
        levelCount: 1,
        baseArrayLayer: 0,
        layerCount: 1,
      },
    }
  }

  pub fn layout(mut self, layout: vk::ImageLayout) -> Self {
    self.layout = layout;
    self
  }

  pub fn clear(mut self, clear: vk::ClearColorValue) -> Self {
    self.clear = clear;
    self
  }

  pub fn subresource(mut self, subresource: vk::ImageSubresourceRange) -> Self {
    self.subresource = subresource;
    self
  }
}

impl StreamPush for ClearColorImage {
  fn enqueue(&self, cs: Stream) -> Stream {
    vk::CmdClearColorImage(cs.buffer, self.image, self.layout, &self.clear, 1, &self.subresource);
    cs
  }
}
