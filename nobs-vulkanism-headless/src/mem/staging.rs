use crate::cmd::commands;
use crate::mem;
use crate::mem::Handle;
use vk;
use vk::builder::Buildable;

pub struct Staging {
  range: StagingRange,
}

impl Drop for Staging {
  fn drop(&mut self) {
    self.range.mem.trash.push_buffer(self.range.buffer);
  }
}

impl std::fmt::Debug for Staging {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(
      f,
      "Staging {{ buffer: {}, offset: {}, size: {}}}",
      self.range.buffer, self.range.offset, self.range.size
    )
  }
}

impl Staging {
  pub fn new(mut mem: mem::Mem, size: vk::DeviceSize) -> Result<Self, mem::Error> {
    let mut buffer = vk::NULL_HANDLE;
    mem::Buffer::new(&mut buffer)
      .size(size)
      .usage(vk::BUFFER_USAGE_TRANSFER_SRC_BIT | vk::BUFFER_USAGE_TRANSFER_DST_BIT | vk::BUFFER_USAGE_UNIFORM_BUFFER_BIT)
      .devicelocal(false)
      .bind(&mut mem.alloc, mem::BindType::Scatter)?;

    Ok(Self {
      range: StagingRange {
        mem,
        buffer,
        offset: 0,
        size,
      },
    })
  }

  pub fn range(&self, offset: vk::DeviceSize, size: vk::DeviceSize) -> StagingRange {
    self.range.range(offset, size)
  }

  pub fn map(&mut self) -> Option<mem::Mapped> {
    self.range.map()
  }

  pub fn copy_from_buffer(&self, src: vk::Buffer, srcoffset: vk::DeviceSize) -> commands::BufferCopy {
    self.range.copy_from_buffer(src, srcoffset)
  }
  pub fn copy_into_buffer(&self, dst: vk::Buffer, dstoffset: vk::DeviceSize) -> commands::BufferCopy {
    self.range.copy_into_buffer(dst, dstoffset)
  }

  pub fn copy_from_image(&self, src: vk::Image, cp: commands::BufferImageCopyBuilder) -> commands::ImageBufferCopy {
    self.range.copy_from_image(src, cp)
  }
  pub fn copy_into_image(&self, dst: vk::Image, cp: commands::BufferImageCopyBuilder) -> commands::BufferImageCopy {
    self.range.copy_into_image(dst, cp)
  }
}

#[derive(Clone)]
pub struct StagingRange {
  mem: mem::Mem,
  buffer: vk::Buffer,
  offset: vk::DeviceSize,
  size: vk::DeviceSize,
}

impl StagingRange {
  pub fn range(&self, offset: vk::DeviceSize, size: vk::DeviceSize) -> Self {
    Self {
      mem: self.mem.clone(),
      buffer: self.buffer,
      offset: self.offset + offset,
      size,
    }
  }

  pub fn map(&mut self) -> Option<mem::Mapped> {
    self
      .mem
      .alloc
      .get_mapped_region(Handle::Buffer(self.buffer), self.offset, self.size)
  }

  pub fn copy_from_buffer(&self, src: vk::Buffer, srcoffset: vk::DeviceSize) -> commands::BufferCopy {
    vk::BufferCopy::build()
      .src_offset(srcoffset)
      .dst_offset(self.offset)
      .size(self.size)
      .copy(src, self.buffer)
  }
  pub fn copy_into_buffer(&self, dst: vk::Buffer, dstoffset: vk::DeviceSize) -> commands::BufferCopy {
    vk::BufferCopy::build()
      .src_offset(dstoffset)
      .dst_offset(self.offset)
      .size(self.size)
      .copy(self.buffer, dst)
  }

  pub fn copy_from_image(&self, src: vk::Image, cp: commands::BufferImageCopyBuilder) -> commands::ImageBufferCopy {
    cp.buffer_offset(self.offset).copy_image_to_buffer(src, self.buffer)
  }
  pub fn copy_into_image(&self, dst: vk::Image, cp: commands::BufferImageCopyBuilder) -> commands::BufferImageCopy {
    cp.buffer_offset(self.offset).copy_buffer_to_image(self.buffer, dst)
  }
}

pub struct StagingFrame {
  stage: Staging,
  range: StagingRange,
}

impl StagingFrame {
  pub fn new(mem: mem::Mem, size: vk::DeviceSize) -> Result<Self, mem::Error> {
    let stage = Staging::new(mem, size)?;
    let range = stage.range.clone();
    Ok(Self { stage, range })
  }

  pub fn next(&mut self, size: vk::DeviceSize) -> Result<StagingRange, mem::Error> {
    if self.range.size < size {
      Err(mem::Error::OutOfMemory)?;
    }
    let r = self.range.range(0, size);
    self.range = self.range.range(size, self.range.size - size);
    Ok(r)
  }

  pub fn reset(&mut self) {
    self.range = self.stage.range.clone();
  }

  pub fn capacity(&self) -> vk::DeviceSize {
    self.stage.range.size
  }
}
