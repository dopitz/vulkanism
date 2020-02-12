use vk;

use crate::Handle;

/// Bundles all information for the [Allocator](struct.Allocator.html) to perform a resource memory binding.
///
/// The size of the resource is determined by `vk::MemoryRequirements`. 
/// Note tha this size may be larger than the actual requested one (e.g. it might be a multiple of 256 bytes for host accassable buffers).
///
/// This needs to be accounted for when mapping the buffer with [Mapped](mapped/struct.Mapped.html).
#[derive(Debug, Clone, Copy)]
pub struct BindInfo {
  pub handle: Handle<u64>,
  pub properties: vk::MemoryPropertyFlags,
  pub linear: bool,
}

impl BindInfo {
  /// Create BindInfo from the specified memory properties
  ///
  /// ## Arguments
  /// *`handle` - the handle that needs a memory binding
  /// *`properties` - the memory properties indicating if the resource is device local or host accessible
  pub fn new(handle: Handle<u64>, properties: vk::MemoryPropertyFlags, linear: bool) -> Self {
    Self {
      handle,
      properties,
      linear,
    }
  }
}

/// Internal bind info used only by PageTable
///
/// Implements conversion from the public [BindInfo](../struct.BindInfo.html).
pub struct BindInfoInner {
  pub handle: Handle<u64>,
  pub requirements: vk::MemoryRequirements,
}

impl BindInfoInner {
  pub fn new(info: &BindInfo, device: vk::Device) -> BindInfoInner {
    let mut requirements = std::mem::MaybeUninit::uninit();
    match info.handle {
      Handle::Image(i) => vk::GetImageMemoryRequirements(device, i, requirements.as_mut_ptr()),
      Handle::Buffer(b) => vk::GetBufferMemoryRequirements(device, b, requirements.as_mut_ptr()),
    }
    let handle = info.handle;
    let requirements = unsafe { requirements.assume_init() };

    Self {
      handle,
      requirements,
    }
  }
}
