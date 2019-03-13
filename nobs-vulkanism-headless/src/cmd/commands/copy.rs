use super::ImageBarrier;
use super::Stream;
use super::StreamPush;
use vk;
use vk::builder::Buildable;

/// Copies memory from one buffer to another
pub struct BufferCopy {
  pub src: vk::Buffer,
  pub dst: vk::Buffer,
  pub region: vk::BufferCopy,
}

impl StreamPush for BufferCopy {
  fn enqueue(&self, cs: Stream) -> Stream {
    vk::CmdCopyBuffer(cs.buffer, self.src, self.dst, 1, &self.region);
    cs
  }
}

pub struct BufferCopyBuilder {
  pub info: vk::BufferCopy,
}

vk_builder!(vk::BufferCopy, BufferCopyBuilder);

impl Default for BufferCopyBuilder {
  fn default() -> Self {
    Self {
      info: vk::BufferCopy {
        size: 0,
        dstOffset: 0,
        srcOffset: 0,
      },
    }
  }
}

impl BufferCopyBuilder {
  pub fn dst_offset(mut self, offset: vk::DeviceSize) -> Self {
    self.info.dstOffset = offset;
    self
  }

  pub fn src_offset(mut self, offset: vk::DeviceSize) -> Self {
    self.info.srcOffset = offset;
    self
  }

  pub fn size(mut self, size: vk::DeviceSize) -> Self {
    self.info.size = size;
    self
  }

  pub fn copy(self, src: vk::Buffer, dst: vk::Buffer) -> BufferCopy {
    BufferCopy {
      src,
      dst,
      region: self.info,
    }
  }
}

/// Copies memory from a buffer to an image
pub struct BufferImageCopy {
  pub src: vk::Buffer,
  pub dst: vk::Image,
  pub region: vk::BufferImageCopy,
}

/// Copies memory from an image to a buffer
pub struct ImageBufferCopy {
  pub src: vk::Image,
  pub dst: vk::Buffer,
  pub region: vk::BufferImageCopy,
}

impl StreamPush for BufferImageCopy {
  fn enqueue(&self, cs: Stream) -> Stream {
    let cs = cs.push(&ImageBarrier::to_transfer_dst(self.dst));
    vk::CmdCopyBufferToImage(
      cs.buffer,
      self.src,
      self.dst,
      vk::IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL,
      1,
      &self.region,
    );
    cs
  }
}

impl StreamPush for ImageBufferCopy {
  fn enqueue(&self, cs: Stream) -> Stream {
    let cs = cs.push(&ImageBarrier::to_transfer_src(self.src));
    vk::CmdCopyBufferToImage(
      cs.buffer,
      self.src,
      self.dst,
      vk::IMAGE_LAYOUT_TRANSFER_SRC_OPTIMAL,
      1,
      &self.region,
    );
    cs
  }
}

pub struct BufferImageCopyBuilder {
  pub info: vk::BufferImageCopy,
}

vk_builder!(vk::BufferImageCopy, BufferImageCopyBuilder);

impl Default for BufferImageCopyBuilder {
  fn default() -> Self {
    Self {
      info: vk::BufferImageCopy {
        bufferOffset: 0,
        bufferRowLength: 0,
        bufferImageHeight: 0,
        imageSubresource: vk::ImageSubresourceLayers::build().layers,
        imageOffset: vk::Offset3D::build().offset,
        imageExtent: vk::Extent3D::build().extent,
      },
    }
  }
}

impl BufferImageCopyBuilder {
  pub fn buffer_offset(mut self, offset: vk::DeviceSize) -> Self {
    self.info.bufferOffset = offset;
    self
  }

  pub fn buffer_rowlenght(mut self, length: u32) -> Self {
    self.info.bufferRowLength = length;
    self
  }

  pub fn buffer_imgheight(mut self, height: u32) -> Self {
    self.info.bufferImageHeight = height;
    self
  }

  pub fn subresource(mut self, layers: vk::ImageSubresourceLayers) -> Self {
    self.info.imageSubresource = layers;
    self
  }

  pub fn image_offset(mut self, offset: vk::Offset3D) -> Self {
    self.info.imageOffset = offset;
    self
  }

  pub fn image_extent(mut self, extent: vk::Extent3D) -> Self {
    self.info.imageExtent = extent;
    self
  }

  pub fn copy_buffer_to_image(self, src: vk::Buffer, dst: vk::Image) -> BufferImageCopy {
    BufferImageCopy {
      src,
      dst,
      region: self.info,
    }
  }

  pub fn copy_image_to_buffer(self, src: vk::Image, dst: vk::Buffer) -> ImageBufferCopy {
    ImageBufferCopy {
      src,
      dst,
      region: self.info,
    }
  }
}

/// Copies memory from one image to another
pub struct ImageCopy {
  pub src: vk::Image,
  pub dst: vk::Image,
  pub region: vk::ImageCopy,
}

impl StreamPush for ImageCopy {
  fn enqueue(&self, cs: Stream) -> Stream {
    let cs = cs
      .push(&ImageBarrier::to_transfer_src(self.src))
      .push(&ImageBarrier::to_transfer_dst(self.dst));
    vk::CmdCopyImage(
      cs.buffer,
      self.src,
      vk::IMAGE_LAYOUT_TRANSFER_SRC_OPTIMAL,
      self.dst,
      vk::IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL,
      1,
      &self.region,
    );
    cs
  }
}

pub struct ImageCopyBuilder {
  pub info: vk::ImageCopy,
}

vk_builder!(vk::ImageCopy, ImageCopyBuilder);

impl Default for ImageCopyBuilder {
  fn default() -> Self {
    Self {
      info: vk::ImageCopy {
        srcSubresource: vk::ImageSubresourceLayers::build().layers,
        srcOffset: vk::Offset3D::build().offset,

        dstSubresource: vk::ImageSubresourceLayers::build().layers,
        dstOffset: vk::Offset3D::build().offset,

        extent: vk::Extent3D::build().extent,
      },
    }
  }
}

impl ImageCopyBuilder {
  pub fn src_subresource(mut self, layers: vk::ImageSubresourceLayers) -> Self {
    self.info.srcSubresource = layers;
    self
  }

  pub fn src_offset(mut self, offset: vk::Offset3D) -> Self {
    self.info.srcOffset = offset;
    self
  }

  pub fn dst_subresource(mut self, layers: vk::ImageSubresourceLayers) -> Self {
    self.info.dstSubresource = layers;
    self
  }

  pub fn dst_offset(mut self, offset: vk::Offset3D) -> Self {
    self.info.dstOffset = offset;
    self
  }

  pub fn extent(mut self, extent: vk::Extent3D) -> Self {
    self.info.extent = extent;
    self
  }

  pub fn copy(self, src: vk::Image, dst: vk::Image) -> ImageCopy {
    ImageCopy {
      src,
      dst,
      region: self.info,
    }
  }
}
