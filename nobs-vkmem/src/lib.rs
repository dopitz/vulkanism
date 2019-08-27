//! Vulkan memory management as extension to [nobs-vk](https://docs.rs/nobs-vk).
//!
//! Buffer and image creation in vulkan is tricky in comparison to e.g. OpenGL, because
//! 1. We have to create the buffer/image and then later bind it to a `vkDeviceMemory` that has to be created separately.
//! 2. Creating a single `vkDeviceMemory` allocation for every buffer/image is bad practice,
//!     in fact it is [encouraged](https://developer.nvidia.com/vulkan-memory-management) to bind resources that are used together on the same allocation.
//! 3. Another layer of difficulty is introduced with memory types, since not all resources (should) share the same memory properties - which is different for each Driver/Vendor
//!
//! nobs-vkmem provides convenient and accessible methods for creating buffers and images and binding them to physical memory.
//! This dramatically reduces boiler plate code, while still offers the user considerable control over how resources are bound to memory.
//! 1. Easy buffer and image creation with builder patterns.
//! 2. Device memory is allocated in larger pages. The crate keeps track of free and used regions in a page.
//! 3. Offers different allocation strategies for different purposes, including forcing the binding of several resources to a continuous block, or binding resources on private pages.
//! 4. Easy mapping of host accessible buffers
//!
//! Interfacing with this crate is mainly handled in [Allocator](struct.Allocator.html), with which buffers and images are bound to device memory.
//!
//! [Buffer](builder/struct.Buffer.html) and [Image](builder/struct.Image.html) provide a convenient way to configure buffers/images and bind them to the allocator in bulk.
//!
//! See [Allocator](struct.Allocator.html) to get a quick overview on how to use create vulkan buffers and images with this crate.
#[macro_use]
extern crate nobs_vk as vk;

mod allocator;
mod bindinfo;
mod bindtype;
mod block;
mod builder;
mod handle;
mod mapped;
mod memtype;
mod table;
mod trash;

pub use allocator::*;
pub use bindinfo::BindInfo;
pub use bindtype::BindType;
pub use builder::Buffer;
pub use builder::Image;
pub use builder::Resource;
pub use handle::Handle;
pub use mapped::Mapped;
pub use memtype::Memtype;
pub use trash::Trash;

/// Errors that can be occure when using this crate
#[derive(Debug)]
pub enum Error {
  /// Indicates, that the desired pagesize is too small.
  /// Pages need to be at least of size `bufferImageGranularity`,
  /// that is defined in the physical device limits (or with [get_min_pagesize](struct.Allocator.html#method.get_min_pagesize)).
  InvalidPageSize,
  /// Indicates, that the allocator could not allocate a new page.
  AllocError,
  /// Indicates, that there is not enough free space available to bind resources.
  OutOfMemory,
  /// Indicates, that this device does not have a memory type, that satisfies a combination of 'vk::MemoryRequirements' and 'vk::MemoryPropertyFlags'.
  InvalidMemoryType,
  /// Indicates, that a buffer create returned unsuccessfull.
  /// The wrapped value is the index of the resource that could not be created.
  CreateBufferFailed(u32),
  /// Indicates, that an image create returned unsuccessfull.
  /// The wrapped value is the index of the resource that could not be created.
  CreateImageFailed(u32),
  /// Indicates, that binding a buffer or image failed
  BindMemoryFailed,
  /// Indicates, that a resource was bound multiple times
  AlreadyBound,
  /// indicates, that the requested memory region could not be mapped
  MapError,
}

/// Wrapper around an [Allocator](struct.Alloctator.html) and [Trash](struct.Trash.html).
#[derive(Clone)]
pub struct Mem {
  pub alloc: Allocator,
  pub trash: Trash,
}

impl Mem {
  /// Create new Mem object from Allocator and Trash
  pub fn new(alloc: Allocator, inflight: usize) -> Self {
    Self {
      alloc: alloc.clone(),
      trash: Trash::new(alloc, inflight)
    }
  }
}

#[macro_export]
macro_rules! device_size {
  ($t:ty) => (
    (std::mem::size_of::<$t>()) as crate::vk::DeviceSize
  );
  ($N:expr, $t:ty) => (
    ($N * std::mem::size_of::<$t>()) as crate::vk::DeviceSize
  );
}
