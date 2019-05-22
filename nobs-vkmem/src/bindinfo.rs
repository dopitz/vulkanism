use vk;

use crate::Handle;

/// Bundles all information for the [Allocator](struct.Allocator.html) to perform a resource memory binding.
///
/// It is possible to additionally specify the size of the resource in bytes ([with_size](struct.BindInfo.html#method.with_size)).
/// This size only matters, if the resource is a buffer that will be mapped into host accessible memory.
/// If no size is specified with the constructor the size for the memory binding will be retrieved from the buffer's `vk::MemoryRequirements`,
/// which might include a padding at the end. If the buffer is then mapped with [get_mapped](struct.Allocator.html#method.get_mapped) the retured
/// [Mapped](mapped/struct.Mapped.html) will contain the buffer including padding. By specifying an explicit size this can be circumvented.
///
/// When the builders [Buffer](builder/struct.Buffer.html) and [Image](builder/struct.Image.html) are used resources will allways be submitted with their actual size.
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
/// The `size` field may be smaller than the `requirements.size`, in case the public BindInfo specified it.
/// Otherwise these two sizes will be equal.
pub struct BindInfoInner {
  pub handle: Handle<u64>,
  pub size: vk::DeviceSize,
  pub requirements: vk::MemoryRequirements,
}

impl BindInfoInner {
  pub fn new(info: &BindInfo, device: vk::Device) -> BindInfoInner {
    let mut requirements = unsafe { std::mem::uninitialized() };
    match info.handle {
      Handle::Image(i) => vk::GetImageMemoryRequirements(device, i, &mut requirements),
      Handle::Buffer(b) => vk::GetBufferMemoryRequirements(device, b, &mut requirements),
    }
    let handle = info.handle;
    let size = requirements.size;

    Self {
      handle,
      size,
      requirements,
    }
  }
}
