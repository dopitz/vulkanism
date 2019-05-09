use crate::block::Block;
use crate::Error;
use vk;

use std::os::raw::c_void;

/// A mapped memory region
///
/// Automatically unmapps the memory when the instance goes out of scope
#[derive(Debug)]
pub struct Mapped {
  device: vk::Device,
  block: Block,
  ptr: *mut c_void,
}

impl Drop for Mapped {
  fn drop(&mut self) {
    vk::UnmapMemory(self.device, self.block.mem);
  }
}

impl Mapped {
  /// Mapps the memory described by `block`
  pub fn new(device: vk::Device, block: Block) -> Result<Mapped, Error> {
    let mut ptr = std::ptr::null_mut();
    vk_check!(vk::MapMemory(
      device,
      block.mem,
      block.beg + block.pad,
      block.size_padded(),
      0,
      &mut ptr
    ))
    .map_err(|_| Error::MapError)?;
    Ok(Mapped { device, block, ptr })
  }

  /// Copies memory from the mapped region on the device to `dst`
  pub fn device_to_host<T>(&self, dst: &mut T) {
    unsafe { std::ptr::copy_nonoverlapping(self.ptr, std::mem::transmute(dst), self.block.size_padded() as usize) };
  }

  /// Copies memory from `src` to the mapped region on the device
  pub fn host_to_device<T>(&self, src: &T) {
    unsafe { std::ptr::copy_nonoverlapping(std::mem::transmute(src), self.ptr, self.block.size_padded() as usize) };
  }

  /// Get a pointer to the mapped memory
  pub fn as_ptr<T>(&self) -> *const T {
    unsafe { std::mem::transmute::<*const c_void, *const T>(self.ptr) }
  }

  /// Get a mutable pointer to the mapped memory
  pub fn as_ptr_mut<T>(&mut self) -> *mut T {
    unsafe { std::mem::transmute::<*mut c_void, *mut T>(self.ptr) }
  }

  /// Get a slice from the mapped memory
  pub fn as_slice<T>(&self) -> &[T] {
    unsafe { std::slice::from_raw_parts(self.as_ptr::<T>(), self.block.size_padded() as usize / std::mem::size_of::<T>()) }
  }

  /// Get a mutable slice from the mapped memory
  pub fn as_slice_mut<T>(&mut self) -> &mut [T] {
    unsafe { std::slice::from_raw_parts_mut(self.as_ptr_mut::<T>(), self.block.size_padded() as usize / std::mem::size_of::<T>()) }
  }

  /// Get the size of the mapped reqion
  pub fn get_size(&self) -> vk::DeviceSize {
    self.block.size_padded()
  }
}
