//! Vulkan memory management as extension to [nobs-vk](https://docs.rs/nobs-vk).
//!
//! Buffer and image creation in vulkan is tricky in comparison to e.g. OpenGL, because
//! 1. We have to create the buffer/image and then later bind it to a `vkDeviceMemory` that has to be created separately.
//! 2. Creating a single `vkDeviceMemory` allocation for every buffer/image is bad practice,
//!     in fact it is [encouraged](https://developer.nvidia.com/vulkan-memory-management) to bind esources that are used together on the same allocation.
//! 3. Another layer of difficulty is introduced with memory types, since not all resources (should) share the same memory properties - which is different for each Driver/Vendor
//!
//! nobs-vkmem provides convenient and accessible methods for creating buffers and images and binding them to physical memory.
//! This dramatically reduces boiler plate code, while still offers the user considerable control over how resources are bound to memory.
//! 1. Easy buffer and image creation with builder patterns.
//! 2. Device memory is allocated in larger pages. The library keeps track of free and used regions in a page.
//! 3. Offers different allocation strategies for different purposes, including forcing the binding of several resources to a continuous block, or binding resources on private pages.
//! 4. Easy mapping of host accessible buffers
//!
//! Interfacing with this library is mainly handled in [Allocator](struct.Allocator.html), with which buffers and images are bound to device memory.
//!
//! [Buffer](builder/struct.Buffer.html) and [Image](builder/struct.Image.html) provide a convenient way to configure buffers/images and bind them to the allocator in bulk.
//!
//! See [Allocator](struct.Allocator.html) to get a quick overview on how to use this library.
#[macro_use]
extern crate nobs_vk as vk;

mod block;
mod builder;
mod mapped;
mod page;

pub use builder::Buffer;
pub use builder::Image;
pub use builder::ImageView;
pub use builder::Resource;
pub use mapped::Mapped;
pub use page::BindType;

use std::collections::HashMap;
use std::fmt::Write;

use block::Block;
use page::PageTable;

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
  /// Indicates that the requested resources can not be bound to a single block, because they are larger than a page.
  OversizedBlock,
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

/// Enum defining resource types that can be bound
///
/// Handles that can be bound to memory are either of type vk::Buffer or vk::Image.
/// In both cases the handles are u64 typedefs.
/// The enum in used, so that we can submit buffer and image handles for binding to the Allocator in a uniform fashion,
/// whlile still being able to distinguish between them.
///
/// Submitting buffers along with images to the [Allocator](struct.Allocator.html) makes sence, when they use the same memory type
/// (which is the case for device local buffers and images)
#[derive(Debug, Clone, Copy)]
pub enum Handle<T>
where
  T: Clone + Copy,
{
  Buffer(T),
  Image(T),
}

impl<T> Handle<T>
where
  T: Clone + Copy,
{
  /// Gets the underlying handle's value
  pub fn get(&self) -> T {
    match self {
      Handle::Image(h) => *h,
      Handle::Buffer(h) => *h,
    }
  }

  /// Convert the value of the handle without changing it's enum type
  fn map<U: Clone + Copy>(self, u: U) -> Handle<U> {
    match self {
      Handle::Image(_) => Handle::Image(u),
      Handle::Buffer(_) => Handle::Buffer(u),
    }
  }
}

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
  handle: Handle<u64>,
  size: Option<vk::DeviceSize>,
  properties: vk::MemoryPropertyFlags,
}

impl BindInfo {
  /// Create BindInfo from the specified memory properties
  ///
  /// ## Arguments
  /// *`handle` - the handle that needs a memory binding
  /// *`properties` - the memory properties indicating if the resource is device local or host accessible
  pub fn new(handle: Handle<u64>, properties: vk::MemoryPropertyFlags) -> Self {
    Self {
      handle,
      properties,
      size: None,
    }
  }

  /// Create BindInfo with the specified memory properties and explicit size in bytes
  ///
  /// ## Arguments
  /// *`handle` - the handle that needs a memory binding
  /// *`size` - the actual size of the resource (in bytes)
  /// *`properties` - the memory properties indicating if the resource is device local or host accessible
  pub fn with_size(handle: Handle<u64>, size: vk::DeviceSize, properties: vk::MemoryPropertyFlags) -> Self {
    Self {
      handle,
      properties,
      size: Some(size),
    }
  }
}

/// Defines meta information for the [Allocator](struct.Allocator.html)
///
/// Caches `vk::MemoryRequirements` for buffers and images.
/// Defines a pagesize for memory types of all common combinations of resource types [buffer, image] and memory properties [device local, host accessible].
#[derive(Debug)]
pub struct AllocatorSizes {
  /// Handle to the physical device
  ///
  /// Used to retrieve and check against device limits
  pub pdevice: vk::PhysicalDevice,
  /// Cached memory requirements for image resourcses
  ///
  /// Used to get the memory type index without having to create an image.
  pub image_requirements: vk::MemoryRequirements,
  /// Cached memory requirements for image resourcses
  ///
  /// Used to get the memory type index without having to create a buffer.
  pub buffer_requirements: vk::MemoryRequirements,

  /// Default page size in bytes
  ///
  /// This is the fallback page size, that is returned in [get_pagesize](struct.AllocatorSizes.html#method.get_pagesize), if no
  /// mapping for the requested memory type index exists.
  ///
  /// The default page size is initialized with 64MiB.
  pub pagesize_default: vk::DeviceSize,
  /// Page size mapped by memory type index
  pub pagesizes: HashMap<u32, vk::DeviceSize>,
}

impl AllocatorSizes {
  /// Creates an AllocatorSizes object with default pagesizes
  ///
  /// Initializes pagesizes for the allocator with
  /// - 128MiB for device local resources
  /// - 8MiB for host accessible resources
  /// - 64MiB fallback pagesize
  ///
  /// # Arguments
  /// * `pdevice` - physical device handle
  /// * `device` - device handle
  ///
  pub fn new(pdevice: vk::PhysicalDevice, device: vk::Device) -> Self {
    let image_requirements = {
      let info = vk::ImageCreateInfo {
        sType: vk::STRUCTURE_TYPE_IMAGE_CREATE_INFO,
        pNext: std::ptr::null(),
        flags: 0,
        imageType: vk::IMAGE_TYPE_2D,
        extent: vk::Extent3D {
          width: 1,
          height: 1,
          depth: 1,
        },
        mipLevels: 1,
        arrayLayers: 1,
        tiling: vk::IMAGE_TILING_OPTIMAL,
        format: vk::FORMAT_R8G8B8A8_SRGB,
        initialLayout: vk::IMAGE_LAYOUT_UNDEFINED,
        usage: vk::IMAGE_USAGE_TRANSFER_DST_BIT | vk::IMAGE_USAGE_SAMPLED_BIT,
        samples: vk::SAMPLE_COUNT_1_BIT,
        sharingMode: vk::SHARING_MODE_EXCLUSIVE,
        queueFamilyIndexCount: 0,
        pQueueFamilyIndices: std::ptr::null(),
      };
      let mut handle = vk::NULL_HANDLE;
      vk_uncheck!(vk::CreateImage(device, &info, std::ptr::null(), &mut handle));

      let mut requirements = unsafe { std::mem::uninitialized() };
      vk::GetImageMemoryRequirements(device, handle, &mut requirements);

      vk::DestroyImage(device, handle, std::ptr::null());

      requirements
    };
    let buffer_requirements = {
      let info = vk::BufferCreateInfo {
        sType: vk::STRUCTURE_TYPE_BUFFER_CREATE_INFO,
        pNext: std::ptr::null(),
        flags: 0,
        size: 1,
        usage: vk::BUFFER_USAGE_TRANSFER_SRC_BIT
          | vk::BUFFER_USAGE_TRANSFER_DST_BIT
          | vk::BUFFER_USAGE_UNIFORM_TEXEL_BUFFER_BIT
          | vk::BUFFER_USAGE_STORAGE_TEXEL_BUFFER_BIT
          | vk::BUFFER_USAGE_UNIFORM_BUFFER_BIT
          | vk::BUFFER_USAGE_STORAGE_BUFFER_BIT
          | vk::BUFFER_USAGE_INDEX_BUFFER_BIT
          | vk::BUFFER_USAGE_VERTEX_BUFFER_BIT
          | vk::BUFFER_USAGE_INDIRECT_BUFFER_BIT,
        sharingMode: vk::SHARING_MODE_EXCLUSIVE,
        queueFamilyIndexCount: 0,
        pQueueFamilyIndices: std::ptr::null(),
      };
      let mut handle = vk::NULL_HANDLE;
      vk_uncheck!(vk::CreateBuffer(device, &info, std::ptr::null(), &mut handle));

      let mut requirements = unsafe { std::mem::uninitialized() };
      vk::GetBufferMemoryRequirements(device, handle, &mut requirements);

      vk::DestroyBuffer(device, handle, std::ptr::null());

      requirements
    };

    let minsize = Allocator::get_min_pagesize(pdevice);
    let pagesize_default = vk::DeviceSize::max(minsize * 4, 1 << 26);
    let mut pagesizes = HashMap::new();

    let memtype_local_image = Allocator::get_memtype(pdevice, &image_requirements, vk::MEMORY_PROPERTY_DEVICE_LOCAL_BIT).unwrap();
    let memtype_local_buffer = Allocator::get_memtype(pdevice, &buffer_requirements, vk::MEMORY_PROPERTY_DEVICE_LOCAL_BIT).unwrap();
    let memtype_hostaccess = Allocator::get_memtype(
      pdevice,
      &buffer_requirements,
      vk::MEMORY_PROPERTY_HOST_VISIBLE_BIT | vk::MEMORY_PROPERTY_HOST_COHERENT_BIT,
    )
    .unwrap();

    pagesizes.insert(memtype_local_image, vk::DeviceSize::max(minsize, 1 << 27));
    pagesizes.insert(memtype_local_buffer, vk::DeviceSize::max(minsize, 1 << 27));
    if memtype_local_buffer != memtype_hostaccess {
      pagesizes.insert(memtype_hostaccess, vk::DeviceSize::max(minsize, 1 << 18));
    }

    Self {
      pagesize_default,
      pagesizes,

      pdevice,
      image_requirements,
      buffer_requirements,
    }
  }

  /// Get the memtype of an image with the specified memory properties
  ///
  /// ## Returns
  ///  - An option with the memory type index.
  ///  - None, if there is no memory type that supports images with the specified properties
  pub fn get_image_memtype(&self, properties: vk::MemoryPropertyFlags) -> Option<u32> {
    Allocator::get_memtype(self.pdevice, &self.image_requirements, properties)
  }

  /// Get the memtype of a buffer with the specified memory properties
  ///
  /// ## Returns
  ///  - An option with the memory type index.
  ///  - None, if there is no memory type that supports buffers with the specified properties
  pub fn get_buffer_memtype(&self, properties: vk::MemoryPropertyFlags) -> Option<u32> {
    Allocator::get_memtype(self.pdevice, &self.buffer_requirements, properties)
  }

  /// Get the pagesize for the memory type
  ///
  /// # Returns
  ///  - The pagesize of the memory type, if one has been set in the AllocatorSizes.
  ///  - Otherwise the default pagesize is returned.
  pub fn get_pagesize(&self, memtype: u32) -> vk::DeviceSize {
    match self.pagesizes.get(&memtype) {
      Some(size) => *size,
      None => self.pagesize_default,
    }
  }

  /// Set the pagesize for the specifid memory type
  ///
  /// The page size must be at least of size `bufferImageGranularity`. If not the result returns [InvalidPageSize](enum.Error.html#field.InvalidPageSize).
  pub fn set_pagesize(&mut self, memtype: u32, size: vk::DeviceSize) -> Result<(), Error> {
    if size < Allocator::get_min_pagesize(self.pdevice) {
      Err(Error::InvalidPageSize)?
    }
    self.pagesizes.entry(memtype).or_insert(size);
    Ok(())
  }
}

/// Allocator for buffer and image resources
///
/// Manages device memory for a single device. The actual memory is managed for each memory type separately by a page table.
/// Pages are allocated lazyly, as soon as memory is needed. The pagesizes may be specified in the [AllocatorSizes](struct.AllocatorSizes.html).
///
/// When the Allocator is dropped, all buffers and allocated device momory is freed.
///
/// # Example
/// The Allocator is created from a device handle and it's associated physical device.
/// Buffers and images can be easyly created with the [Buffer](builder/struct.Buffer.html) and [Image](builder/struct.Image.html) builder.
/// Host accessible buffers can be conveniently accessed with [get_mapped / get_mapped_region](struct.Allocator.html#method.get_mapped) and [Mapped](mapped/struct.Mapped.html).
/// Allocated resources may be freed with [destroy(_many)](struct.Allocator.html#method.destroy).
/// Memory of pages that have no bore resource binding can be freed with [free_unused](struct.Allocator.html#method.free_unused) again.
///
/// ```rust
/// extern crate nobs_vk as vk;
/// extern crate nobs_vkmem as vkmem;
///
/// struct Ub {
///   a: u32,
///   b: u32,
///   c: u32,
/// }
///
/// # fn main() {
/// #  let lib = vk::VkLib::new();
/// #  let inst = vk::instance::new()
/// #    .validate(vk::DEBUG_REPORT_ERROR_BIT_EXT | vk::DEBUG_REPORT_WARNING_BIT_EXT)
/// #    .application("awesome app", 0)
/// #    .add_extension(vk::KHR_SURFACE_EXTENSION_NAME)
/// #    .add_extension(vk::KHR_XLIB_SURFACE_EXTENSION_NAME)
/// #    .create(lib)
/// #    .unwrap();
/// #  let (pdevice, device) = vk::device::PhysicalDevice::enumerate_all(inst.handle)
/// #    .remove(0)
/// #    .into_device()
/// #    .add_queue(vk::device::QueueProperties {
/// #      present: false,
/// #      graphics: true,
/// #      compute: true,
/// #      transfer: true,
/// #    }).create()
/// #    .unwrap();
///
/// // create an allocator with default page size
/// // (128MiB for device local / 8MB for host visible memory)
/// let mut allocator = vkmem::Allocator::new(pdevice.handle, device.handle);
///
/// // declare handles
/// let mut buf_ub = vk::NULL_HANDLE;
/// let mut buf_out = vk::NULL_HANDLE;
/// let mut img = vk::NULL_HANDLE;
/// let mut bb = vk::NULL_HANDLE;
///
/// // configure create infos
/// vkmem::Buffer::new(&mut buf_ub)
///   .size(std::mem::size_of::<Ub>() as vk::DeviceSize)
///   .usage(vk::BUFFER_USAGE_TRANSFER_DST_BIT | vk::BUFFER_USAGE_UNIFORM_BUFFER_BIT)
///   .devicelocal(false)
///   .new_buffer(&mut buf_out)
///   .size(123 * std::mem::size_of::<u32>() as vk::DeviceSize)
///   .usage(vk::BUFFER_USAGE_TRANSFER_DST_BIT | vk::BUFFER_USAGE_STORAGE_BUFFER_BIT)
///   .devicelocal(false)
///   .new_image(&mut img)
///   .image_type(vk::IMAGE_TYPE_2D)
///   .size(123, 123, 1)
///   .usage(vk::IMAGE_USAGE_SAMPLED_BIT)
///   .devicelocal(true)
///   .new_buffer(&mut bb)
///   .size(123 * std::mem::size_of::<u32>() as vk::DeviceSize)
///   .usage(vk::BUFFER_USAGE_TRANSFER_DST_BIT | vk::BUFFER_USAGE_STORAGE_BUFFER_BIT)
///   .devicelocal(true)
///
///   // this will create the image/buffers and bind them to memory
///   .bind(&mut allocator, vkmem::BindType::Scatter)
///   .unwrap();
///
/// // Mapped gives a convenient view on the memory
/// // get_mapped mapps the whole block of memory to which the resources was bound
/// // get_mapped_region lets us define a byte offset respective to the beginning of the resource and a size in bytes
/// {
///   let mapped = allocator.get_mapped(buf_ub).unwrap();
///   let ubb = Ub { a: 123, b: 4, c: 5 };
///   mapped.host_to_device(&ubb);
/// }
/// {
///   let mapped = allocator.get_mapped(buf_ub).unwrap();
///   let mut ubb = Ub { a: 0, b: 0, c: 0 };
///   mapped.device_to_host(&mut ubb);
/// }
///
/// {
///   let mapped = allocator.get_mapped_region(buf_out, 4, 100).unwrap();
///   let v = mapped.as_slice::<u32>();
/// }
///
/// // we can print stats in a yaml format for the currently allocated pages
/// println!("{}", allocator.print_stats());
///
/// // buffers and images can be destroyed
/// allocator.destroy(img);
/// allocator.destroy_many(&[buf_ub, buf_out]);
///
/// // destroying does NOT free memory
/// // if we want to free memory we can do this only if a whole page is not used any more
/// // in this case we can free the memory again
/// allocator.free_unused();
///
/// // dropping the allocator automatically destroys bound resources and frees all memory
/// # }
/// ```
pub struct Allocator {
  device: vk::Device,
  sizes: AllocatorSizes,

  pagetbls: HashMap<u32, page::PageTable>,
  handles: HashMap<u64, Handle<u32>>,
}

impl Drop for Allocator {
  fn drop(&mut self) {
    for (h, mt) in self.handles.iter() {
      match mt {
        Handle::Buffer(_) => vk::DestroyBuffer(self.device, *h, std::ptr::null()),
        Handle::Image(_) => vk::DestroyImage(self.device, *h, std::ptr::null()),
      };
    }
  }
}

impl Allocator {
  /// Gets the smallest pagesize for the specified physical device
  ///
  /// The minimum page size is defined through the `bufferImageGranularity` of the physical device properties
  ///
  /// # Arguments
  /// * `pdevice` - physical device handle
  ///
  /// # Returns
  /// The minimum pagesize in bytes.
  pub fn get_min_pagesize(pdevice: vk::PhysicalDevice) -> vk::DeviceSize {
    let mut properties: vk::PhysicalDeviceProperties = unsafe { std::mem::uninitialized() };
    vk::GetPhysicalDeviceProperties(pdevice, &mut properties);
    properties.limits.bufferImageGranularity
  }

  /// Get the memtype for a combination of memory requirements and properties
  ///
  /// # Arguments
  /// * `pdevice` - physical device handle
  /// * `requirements` - memory reqirements retrieved with `vk::GetBufferMemoryRequirements` or `vk::GetBufferMemoryRequirements`
  /// * `properties` - combination of `vk::MemoryPropertyFlagBits`
  ///
  /// # Returns
  /// * `Some(memtype)` - where `memtype` is the memory type index
  /// * `None` - if there no such a combination of memory requirements and properties exists on the physical device
  pub fn get_memtype(
    pdevice: vk::PhysicalDevice,
    requirements: &vk::MemoryRequirements,
    properties: vk::MemoryPropertyFlags,
  ) -> Option<u32> {
    let mut device_properties = unsafe { std::mem::uninitialized() };
    vk::GetPhysicalDeviceMemoryProperties(pdevice, &mut device_properties);

    (0..device_properties.memoryTypeCount)
      .into_iter()
      .position(|i| {
        (requirements.memoryTypeBits & (1 << i)) != 0
          && (device_properties.memoryTypes[i as usize].propertyFlags & properties) == properties
      })
      .map(|i| i as u32)
  }

  /// Creates an Allocator
  ///
  /// The Allocator will use the default AllocatorSizes, with 128MiB / 8MiB pagesizes for device local / host accessible memory.
  ///
  /// # Arguments
  /// * `pdevice` - physical device handle
  /// * `device` - device handle
  pub fn new(pdevice: vk::PhysicalDevice, device: vk::Device) -> Allocator {
    Self::with_sizes(device, AllocatorSizes::new(pdevice, device))
  }

  /// Creates an Allocator from AllocatorSizes
  ///
  /// The AllocatorSizes will consumed by the consturctor.
  /// This allows additional configuration of the pagesizes for every memory type.
  ///
  /// # Arguments
  /// * `device` - device handle
  /// * `sizes` - AllocatorSizes with the physical device handle and pagesize definitions
  ///
  /// # Example
  /// Creates an Allocator with 32MiB large pages for host accassable memory
  /// ```rust
  /// # extern crate nobs_vk as vk;
  /// # extern crate nobs_vkmem as vkmem;
  /// # fn main() {
  /// #  let lib = vk::VkLib::new();
  /// #  let inst = vk::instance::new()
  /// #    .validate(vk::DEBUG_REPORT_ERROR_BIT_EXT | vk::DEBUG_REPORT_WARNING_BIT_EXT)
  /// #    .application("awesome app", 0)
  /// #    .add_extension(vk::KHR_SURFACE_EXTENSION_NAME)
  /// #    .add_extension(vk::KHR_XLIB_SURFACE_EXTENSION_NAME)
  /// #    .create(lib)
  /// #    .unwrap();
  /// #  let (pdevice, device) = vk::device::PhysicalDevice::enumerate_all(inst.handle)
  /// #    .remove(0)
  /// #    .into_device()
  /// #    .add_queue(vk::device::QueueProperties {
  /// #      present: false,
  /// #      graphics: true,
  /// #      compute: true,
  /// #      transfer: true,
  /// #    }).create()
  /// #    .unwrap();
  /// let mut sizes = vkmem::AllocatorSizes::new(pdevice.handle, device.handle);
  /// if let Some(memtype) = sizes.get_buffer_memtype(vk::MEMORY_PROPERTY_HOST_VISIBLE_BIT | vk::MEMORY_PROPERTY_HOST_COHERENT_BIT) {
  ///   sizes.set_pagesize(memtype, 1 << 25);
  /// }
  /// let mut allocator = vkmem::Allocator::with_sizes(pdevice.handle, sizes);
  /// # }
  /// ```
  pub fn with_sizes(device: vk::Device, sizes: AllocatorSizes) -> Allocator {
    Allocator {
      device,
      sizes,
      pagetbls: Default::default(),
      handles: Default::default(),
    }
  }

  /// Bind resources to device memory
  ///
  /// This will automatically allocate memory, if necessary.
  ///
  /// Resources that are used together shoulb be bound in the same call to this function,
  /// since it tries to minimize the number of continuous memory blocks on which the resources are bound.
  /// Predominatly this is important after resources have been deleted and free blocks of memory are distributed over multiple pages.
  ///
  /// We can specify a rough strategy of how resources are bound with the `bindtype` parameter. See [BindType](page/enum.BindType.html) for the different strategies
  ///
  /// The Allocator takes ownorship ower the resource handles.
  ///
  /// Note that it is not neccesary that all [BindInfos](struct.BindInfo.html) need to specify resources that are allocated on the same memory type.
  /// The Allocator will detangle the BindInfos automatically and generate one binding for every group of resources on the same memory type.
  ///
  /// # Arguments
  /// * `bindinfos` - Array of [BindInfo](struct.BindInfo.html) that specifies the resources to be bound
  /// * `bindtype` - Allocation strategy
  ///
  /// # Returns
  /// [Error](enum.Error.html) if the allocator failed to bind one or more resources:
  /// * `Error::AllocError` - if a new page could not be allocated
  /// * `Error::OutOfMemory` - if not enough memory is available to bind all resources
  /// * `Error::OversizedBlock` - if a single resourcse is larger than it's associated pagesize,
  ///                             or if `bindtype` is `BindType::Block` and the combined size of all resources is larger than their associated pagesize
  /// * `Error::InvalidMemoryType` - if for one or more resources no memory type is found that satisfies the memory requirements and properties
  /// * `Error::BindMemoryFailed` - if one or more resources could not be bound to device memory
  /// * `Error::AlreadyBound` - if one or more resources are already bound to device memory
  ///
  /// # Example
  /// Shows how buffers and images can be bound to the allocator. Prior to bind the resources have to be created with either `vk::CreateBuffer` or `vk::CreateImage`.
  ///
  /// The BindInfo can be generated manually (as in the example),
  /// see [Buffer](builder/struct.Buffer.html) and [Image](builder/struct.Image.html) for convenient configuration and binding of buffers and images
  ///
  /// ```rust,no_run
  /// # extern crate nobs_vk as vk;
  /// # extern crate nobs_vkmem as vkmem;
  /// # fn main() {
  /// #  let lib = vk::VkLib::new();
  /// #  let inst = vk::instance::new()
  /// #    .validate(vk::DEBUG_REPORT_ERROR_BIT_EXT | vk::DEBUG_REPORT_WARNING_BIT_EXT)
  /// #    .application("awesome app", 0)
  /// #    .create(lib)
  /// #    .unwrap();
  /// #  let (pdevice, device) = vk::device::PhysicalDevice::enumerate_all(inst.handle)
  /// #    .remove(0)
  /// #    .into_device()
  /// #    .add_queue(vk::device::QueueProperties {
  /// #      present: false,
  /// #      graphics: true,
  /// #      compute: true,
  /// #      transfer: true,
  /// #    }).create()
  /// #    .unwrap();
  /// let mut allocator = vkmem::Allocator::new(pdevice.handle, device.handle);
  ///
  /// let buf = vk::NULL_HANDLE;
  /// let img = vk::NULL_HANDLE;
  /// // ... create buffers/images
  ///
  /// let buf = vkmem::Handle::Buffer(buf);
  /// let img = vkmem::Handle::Image(img);
  /// let host_access_flags =
  ///     vk::MEMORY_PROPERTY_HOST_VISIBLE_BIT |
  ///     vk::MEMORY_PROPERTY_HOST_COHERENT_BIT;
  /// let device_local_flags = vk::MEMORY_PROPERTY_DEVICE_LOCAL_BIT;
  ///
  /// allocator.bind(
  ///   &[
  ///     vkmem::BindInfo::with_size(buf, 12, host_access_flags),
  ///     vkmem::BindInfo::new(img, device_local_flags),
  ///   ],
  ///   vkmem::BindType::Scatter,
  /// ).expect("binding buffers failed");
  /// # }
  /// ```
  pub fn bind(&mut self, bindinfos: &[BindInfo], bindtype: BindType) -> Result<(), Error> {
    if bindinfos.is_empty() {
      return Ok(());
    }

    // sort handles into groups of the same memory type
    let mut by_memtype = HashMap::new();
    for info in bindinfos.iter() {
      let pageinfo = page::BindInfo::new(self.device, info);
      let memtype = Self::get_memtype(self.sizes.pdevice, &pageinfo.requirements, info.properties).ok_or(Error::InvalidMemoryType)?;
      by_memtype.entry(memtype).or_insert(Vec::new()).push(pageinfo);
    }

    // for every group with the same memtype bind the buffers to a page table
    let device = self.device;
    for (memtype, infos) in by_memtype {
      let pagesize = self.sizes.get_pagesize(memtype);
      self
        .pagetbls
        .entry(memtype)
        .or_insert_with(|| PageTable::new(device, memtype, pagesize))
        .bind(&infos, bindtype)?;

      for h in infos.iter().map(|i| i.handle) {
        self.handles.insert(h.get(), h.map(memtype));
      }
    }

    Ok(())
  }

  /// Destroys a resource, that has been bound to this allocator
  ///
  /// see [destroy_many](struct.Allocator.html#method.destroy_many)
  pub fn destroy(&mut self, handle: u64) {
    self.destroy_many(&[handle]);
  }

  /// Destroys resources, that have been bound to this allocator
  ///
  /// Destroys the buffer or image and makes their associated memory available again.
  ///
  /// Freeing the memory is relatively expensive compared to allocation, which is why it should be preferred to use `destroy_many` over just `destroy`.
  /// When multiple resources are destroyed in bulk, merging blocks of free memory and padding together with the freed allocation has to be done only once.
  ///
  /// Note that memory bindings of not destroyed resources can not be rearranged, since vulkan does not allow rebinding a buffer/image to a different location.
  ///
  /// # Example
  /// ```rust
  /// # extern crate nobs_vk as vk;
  /// # extern crate nobs_vkmem as vkmem;
  /// # fn main() {
  /// #  let lib = vk::VkLib::new();
  /// #  let inst = vk::instance::new()
  /// #    .validate(vk::DEBUG_REPORT_ERROR_BIT_EXT | vk::DEBUG_REPORT_WARNING_BIT_EXT)
  /// #    .application("awesome app", 0)
  /// #    .create(lib)
  /// #    .unwrap();
  /// #  let (pdevice, device) = vk::device::PhysicalDevice::enumerate_all(inst.handle)
  /// #    .remove(0)
  /// #    .into_device()
  /// #    .add_queue(vk::device::QueueProperties {
  /// #      present: false,
  /// #      graphics: true,
  /// #      compute: true,
  /// #      transfer: true,
  /// #    }).create()
  /// #    .unwrap();
  /// let mut allocator = vkmem::Allocator::new(pdevice.handle, device.handle);
  ///
  /// let mut buf = vk::NULL_HANDLE;
  /// let mut img = vk::NULL_HANDLE;
  /// # vkmem::Buffer::new(&mut buf)
  /// #   .size(12)
  /// #   .usage(vk::BUFFER_USAGE_TRANSFER_DST_BIT | vk::BUFFER_USAGE_UNIFORM_BUFFER_BIT)
  /// #   .devicelocal(false)
  /// #   .new_image(&mut img)
  /// #   .image_type(vk::IMAGE_TYPE_2D)
  /// #   .size(123, 123, 1)
  /// #   .usage(vk::IMAGE_USAGE_SAMPLED_BIT)
  /// #   .bind(&mut allocator, vkmem::BindType::Scatter)
  /// #   .unwrap();
  /// //... create, bind, use ...
  /// allocator.destroy_many(&[buf, img]);
  /// # }
  /// ```
  pub fn destroy_many(&mut self, handles: &[u64]) {
    let by_memtype = handles
      .iter()
      .filter_map(|h| self.handles.get(h).map(|mt| (h, mt.get())))
      .fold(HashMap::new(), |mut acc, (h, mt)| {
        acc.entry(mt).or_insert(Vec::new()).push(*h);
        acc
      });

    for (mt, hs) in by_memtype.iter() {
      self.pagetbls.get_mut(mt).unwrap().unbind(hs);
    }

    for h in handles.iter() {
      match self.handles.remove(h) {
        Some(Handle::Buffer(_)) => vk::DestroyBuffer(self.device, *h, std::ptr::null()),
        Some(Handle::Image(_)) => vk::DestroyImage(self.device, *h, std::ptr::null()),
        None => (),
      };
    }
  }

  /// Frees memory of unused pages
  pub fn free_unused(&mut self) {
    for (_, tbl) in self.pagetbls.iter_mut() {
      tbl.free_unused();
    }
  }

  /// Gets the physical device handle
  pub fn get_physical_device(&self) -> vk::PhysicalDevice {
    self.sizes.pdevice
  }

  /// Gets the device handle
  pub fn get_device(&self) -> vk::Device {
    self.device
  }

  fn get_mem(&self, handle: u64) -> Option<Block> {
    self
      .handles
      .get(&handle)
      .and_then(|t| self.pagetbls.get(&t.get()))
      .and_then(|tbl| tbl.get_mem(handle))
  }

  /// Gets a [Mapped](mapped/struct.Mapped.html) of the spicified resource handle
  ///
  /// # Example
  /// Creates a uniform buffer, stores some values in it and reads them back
  ///```rust
  /// # extern crate nobs_vk as vk;
  /// # extern crate nobs_vkmem as vkmem;
  /// # fn main() {
  /// #  let lib = vk::VkLib::new();
  /// #  let inst = vk::instance::new()
  /// #    .validate(vk::DEBUG_REPORT_ERROR_BIT_EXT | vk::DEBUG_REPORT_WARNING_BIT_EXT)
  /// #    .application("awesome app", 0)
  /// #    .create(lib)
  /// #    .unwrap();
  /// #  let (pdevice, device) = vk::device::PhysicalDevice::enumerate_all(inst.handle)
  /// #    .remove(0)
  /// #    .into_device()
  /// #    .add_queue(vk::device::QueueProperties {
  /// #      present: false,
  /// #      graphics: true,
  /// #      compute: true,
  /// #      transfer: true,
  /// #    }).create()
  /// #    .unwrap();
  /// let mut allocator = vkmem::Allocator::new(pdevice.handle, device.handle);
  ///
  /// #[derive(Debug)]
  /// struct Ub { a: u32, b: u32, c: u32 }
  /// let mut buf_ub = vk::NULL_HANDLE;
  ///
  /// vkmem::Buffer::new(&mut buf_ub)
  ///   .size(std::mem::size_of::<Ub>() as vk::DeviceSize)
  ///   .usage(vk::BUFFER_USAGE_TRANSFER_DST_BIT | vk::BUFFER_USAGE_UNIFORM_BUFFER_BIT)
  ///   .devicelocal(false)
  ///   .bind(&mut allocator, vkmem::BindType::Scatter)
  ///   .unwrap();
  ///
  /// {
  ///   let mapped = allocator.get_mapped(buf_ub).unwrap();
  ///   let ubb = Ub { a: 123, b: 4, c: 5 };
  ///   mapped.host_to_device(&ubb);
  /// }
  /// {
  ///   let mapped = allocator.get_mapped(buf_ub).unwrap();
  ///   let mut ubb = Ub { a: 0, b: 0, c: 0 };
  ///   mapped.device_to_host(&mut ubb);
  ///   assert_eq!(ubb.a, 123);
  ///   assert_eq!(ubb.b, 4);
  ///   assert_eq!(ubb.c, 5);
  /// }
  /// # }
  /// ```
  pub fn get_mapped(&self, handle: u64) -> Option<Mapped> {
    self.get_mem(handle).and_then(|b| Mapped::new(self.device, b).ok())
  }

  /// Gets a [Mapped](mapped/struct.Mapped.html) of the spicified resource handle with an offset and size in bytes
  ///
  /// # Example
  /// Creates a `u32` shader storage buffer, mappes it with offset and writes and reads values
  ///
  ///```rust
  /// # extern crate nobs_vk as vk;
  /// # extern crate nobs_vkmem as vkmem;
  /// # fn main() {
  /// #  let lib = vk::VkLib::new();
  /// #  let inst = vk::instance::new()
  /// #    .validate(vk::DEBUG_REPORT_ERROR_BIT_EXT | vk::DEBUG_REPORT_WARNING_BIT_EXT)
  /// #    .application("awesome app", 0)
  /// #    .create(lib)
  /// #    .unwrap();
  /// #  let (pdevice, device) = vk::device::PhysicalDevice::enumerate_all(inst.handle)
  /// #    .remove(0)
  /// #    .into_device()
  /// #    .add_queue(vk::device::QueueProperties {
  /// #      present: false,
  /// #      graphics: true,
  /// #      compute: true,
  /// #      transfer: true,
  /// #    }).create()
  /// #    .unwrap();
  /// let mut allocator = vkmem::Allocator::new(pdevice.handle, device.handle);
  ///
  /// let mut buf = vk::NULL_HANDLE;
  /// vkmem::Buffer::new(&mut buf)
  ///   .size(123 * std::mem::size_of::<u32>() as vk::DeviceSize)
  ///   .usage(vk::BUFFER_USAGE_TRANSFER_DST_BIT | vk::BUFFER_USAGE_STORAGE_BUFFER_BIT)
  ///   .devicelocal(false)
  ///   .bind(&mut allocator, vkmem::BindType::Scatter)
  ///   .unwrap();
  ///
  /// {
  ///   let mut mapped = allocator.get_mapped_region(buf, 4, 100).unwrap();
  ///   let v = mapped.as_slice_mut::<u32>();
  ///   v[0] = 123;
  ///   v[1] = 4;
  ///   v[2] = 5;
  /// }
  /// {
  ///   let mapped = allocator.get_mapped_region(buf, 4, 100).unwrap();
  ///   let v = mapped.as_slice::<u32>();
  ///   assert_eq!(v[0], 123);
  ///   assert_eq!(v[1], 4);
  ///   assert_eq!(v[2], 5);
  /// }
  /// # }
  /// ```
  pub fn get_mapped_region(&self, handle: u64, offset: vk::DeviceSize, size: vk::DeviceSize) -> Option<Mapped> {
    self.get_mem(handle).and_then(|b| {
      let region = Block::new(b.begin + offset, b.begin + offset + size, b.mem);
      match region.begin < b.end && region.end <= b.end {
        true => Mapped::new(self.device, region).ok(),
        false => None,
      }
    })
  }

  /// Print staticstics for the Allocator in yaml format
  pub fn print_stats(&self) -> String {
    let mut s = String::new();

    let mut keys: Vec<_> = self.pagetbls.keys().collect();
    keys.sort();
    for k in keys {
      write!(s, "{}", self.pagetbls[k].print_stats()).unwrap();
    }
    s
  }
}
