use crate::cmd::commands;
use crate::mem;
use vk;
use vk::builder::Buildable;

pub struct Staging {
  mem: mem::Mem,
  pub buffer: vk::Buffer,
  pub offset: vk::DeviceSize,
  pub size: vk::DeviceSize,
}

impl Drop for Staging {
  fn drop(&mut self) {
    self.mem.trash.push(self.buffer);
  }
}

impl std::fmt::Debug for Staging {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(
      f,
      "Staging {{ buffer: {}, offset: {}, size: {}}}",
      self.buffer, self.offset, self.size
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
      mem,
      buffer,
      offset: 0,
      size,
    })
  }

  pub fn range(&self, offset: vk::DeviceSize, size: vk::DeviceSize) -> Self {
    Self {
      mem: self.mem.clone(),
      buffer: self.buffer,
      offset: self.offset + offset,
      size,
    }
  }


  pub fn map(&mut self) -> Option<mem::Mapped> {
    self.mem.alloc.get_mapped_region(self.buffer, self.offset, self.size)
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
}

impl StagingFrame {
  pub fn new(stage: Staging) -> Self {
    Self { stage }
  }

  pub fn next(&mut self, size: vk::DeviceSize) -> Result<Staging, mem::Error> {
    if self.stage.size < size {
      Err(mem::Error::OutOfMemory)?;
    }
    let s = self.stage.range(0, size);
    self.stage = self.stage.range(size, self.stage.size - size);
    Ok(s)
  }
}
