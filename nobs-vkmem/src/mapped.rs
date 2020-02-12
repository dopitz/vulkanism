use crate::block::Block;
use crate::Error;
use vk;

/// A mapped memory region
///
/// Automatically unmapps the memory when the instance goes out of scope
#[derive(Debug)]
pub struct Mapped {
  device: vk::Device,
  block: Block,
  ptr: *mut u8,
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
    Ok(Mapped {
      device,
      block,
      ptr: unsafe { std::mem::transmute(ptr) },
    })
  }

  /// Copies memory from the mapped region on the device to `dst`
  pub fn device_to_host<T>(&self) -> T {
    unsafe {
      let mut dst = std::mem::MaybeUninit::<T>::uninit();
      std::ptr::copy_nonoverlapping(self.ptr, std::mem::transmute(dst.as_mut_ptr()), std::mem::size_of::<T>());
      dst.assume_init()
    }
  }
  pub fn device_to_host_slice<T>(&self, dst: &mut [T]) {
    unsafe { std::ptr::copy_nonoverlapping(self.ptr, std::mem::transmute(dst.as_mut_ptr()), std::mem::size_of::<T>() * dst.len()) };
  }

  /// Copies memory from `src` to the mapped region on the device
  pub fn host_to_device<T>(&self, src: &T) {
    unsafe { std::ptr::copy_nonoverlapping(std::mem::transmute(src), self.ptr, std::mem::size_of::<T>()) };
  }
  /// Copies memory from `src` to the mapped region on the device
  pub fn host_to_device_slice<T>(&self, src: &[T]) {
    unsafe { std::ptr::copy_nonoverlapping(std::mem::transmute(src.as_ptr()), self.ptr, std::mem::size_of::<T>() * src.len()) };
  }

  /// Get a pointer to the mapped memory
  pub fn as_ptr<T>(&self) -> *const T {
    unsafe { std::mem::transmute(self.ptr) }
  }

  /// Get a mutable pointer to the mapped memory
  pub fn as_ptr_mut<T>(&mut self) -> *mut T {
    unsafe { std::mem::transmute(self.ptr) }
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
