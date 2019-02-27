use crate::mem;
use vk;

pub struct Staging<'a> {
  alloc: &'a mut mem::Allocator,
  buffer: vk::Buffer,
}

impl<'a> Drop for Staging<'a> {
  fn drop(&mut self) {
    self.alloc.destroy(self.buffer);
  }
}

impl<'a> Staging<'a> {
  pub fn new(alloc: &'a mut mem::Allocator, size: vk::DeviceSize) -> Result<Self, mem::Error> {
    let mut buffer = vk::NULL_HANDLE;
    mem::Buffer::new(&mut buffer)
      .size(size)
      .usage(vk::BUFFER_USAGE_TRANSFER_SRC_BIT | vk::BUFFER_USAGE_TRANSFER_DST_BIT | vk::BUFFER_USAGE_UNIFORM_BUFFER_BIT)
      .devicelocal(false)
      .bind(alloc, mem::BindType::Scatter)?;

    Ok(Self { alloc, buffer })
  }
}
