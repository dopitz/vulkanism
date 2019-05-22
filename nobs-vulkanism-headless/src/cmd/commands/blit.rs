use super::Stream;
use super::StreamPush;
use vk;

/// Blit command for copying, scaling and filtering an image
#[derive(Clone, Copy)]
pub struct Blit {
  pub region: vk::ImageBlit,
  pub im_src: vk::Image,
  pub im_dst: vk::Image,
  pub im_filter: vk::Filter,
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
      im_src: vk::NULL_HANDLE,
      im_dst: vk::NULL_HANDLE,
      im_filter: vk::FILTER_LINEAR,
    }
  }

  pub fn src(mut self, img: vk::Image) -> Self {
    self.im_src = img;
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
    self.im_dst = img;
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
    self.im_filter = filter;
    self
  }
}

impl StreamPush for Blit {
  fn enqueue(&self, cs: Stream) -> Stream {
    vk::CmdBlitImage(
      cs.buffer,
      self.im_src,
      vk::IMAGE_LAYOUT_TRANSFER_SRC_OPTIMAL,
      self.im_dst,
      vk::IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL,
      1,
      &self.region,
      self.im_filter,
    );
    cs
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
