use super::Stream;
use super::StreamPush;
use vk;

/// Copies memory from one buffer to another
pub struct BufferCopy {
  pub src: vk::Buffer,
  pub dst: vk::Buffer,
  pub region: vk::BufferCopy,
}

impl Default for BufferCopy {
  fn default() -> Self {
    BufferCopy {
      src: vk::NULL_HANDLE,
      dst: vk::NULL_HANDLE,
      region: vk::BufferCopy {
        size: 0,
        dstOffset: 0,
        srcOffset: 0,
      },
    }
  }
}

impl BufferCopy {
  pub fn new(src: vk::Buffer, dst: vk::Buffer, region: vk::BufferCopy) -> BufferCopy {
    BufferCopy { src, dst, region }
  }

  pub fn from(mut self, src: vk::Buffer) -> Self {
    self.src = src;
    self
  }
  pub fn from_offset(mut self, src: vk::Buffer, offset: vk::DeviceSize) -> Self {
    self.src = src;
    self.region.srcOffset = offset;
    self
  }

  pub fn to(mut self, src: vk::Buffer) -> Self {
    self.src = src;
    self
  }
  pub fn to_offset(mut self, dst: vk::Buffer, offset: vk::DeviceSize) -> Self {
    self.dst = dst;
    self.region.dstOffset = offset;
    self
  }

  pub fn size(mut self, size: vk::DeviceSize) -> Self {
    self.region.size = size;
    self
  }
}

impl StreamPush for BufferCopy {
  fn enqueue(&self, cs: Stream) -> Stream{
    vk::CmdCopyBuffer(cs.buffer, self.src, self.dst, 1, &self.region);
    cs
  }
}
