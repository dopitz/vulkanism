use std::collections::HashMap;
use std::fmt::Write;

use crate::bindinfo::BindInfoInner;
use crate::block::Block;
use crate::table::Table;
use crate::BindInfo;
use crate::BindType;
use crate::Error;
use crate::Handle;
use crate::Mapped;
use crate::Memtype;

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
  pub pagesizes: HashMap<Memtype, vk::DeviceSize>,
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

      let mut requirements = std::mem::MaybeUninit::zeroed();
      vk::GetImageMemoryRequirements(device, handle, requirements.as_mut_ptr());
      vk::DestroyImage(device, handle, std::ptr::null());

      unsafe { requirements.assume_init() }
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

      let mut requirements = std::mem::MaybeUninit::zeroed();
      vk::GetBufferMemoryRequirements(device, handle, requirements.as_mut_ptr());
      vk::DestroyBuffer(device, handle, std::ptr::null());

      unsafe { requirements.assume_init() }
    };

    let minsize = Allocator::get_min_pagesize(pdevice);
    let pagesize_default = vk::DeviceSize::max(minsize * 4, 1 << 26);
    let mut pagesizes = HashMap::new();

    let memtype_idx_local_image = Allocator::get_memtype(pdevice, &image_requirements, vk::MEMORY_PROPERTY_DEVICE_LOCAL_BIT).unwrap();
    let memtype_idx_local_buffer = Allocator::get_memtype(pdevice, &buffer_requirements, vk::MEMORY_PROPERTY_DEVICE_LOCAL_BIT).unwrap();
    let memtype_idx_hostaccess = Allocator::get_memtype(
      pdevice,
      &buffer_requirements,
      vk::MEMORY_PROPERTY_HOST_VISIBLE_BIT | vk::MEMORY_PROPERTY_HOST_COHERENT_BIT,
    )
    .unwrap();

    pagesizes.insert(
      Memtype {
        index: memtype_idx_local_image,
        linear: false,
      },
      vk::DeviceSize::max(minsize, 1 << 27),
    );
    pagesizes.insert(
      Memtype {
        index: memtype_idx_local_image,
        linear: true,
      },
      vk::DeviceSize::max(minsize, 1 << 27),
    );
    pagesizes.insert(
      Memtype {
        index: memtype_idx_local_buffer,
        linear: true,
      },
      vk::DeviceSize::max(minsize, 1 << 27),
    );
    if memtype_idx_local_buffer != memtype_idx_hostaccess {
      pagesizes.insert(
        Memtype {
          index: memtype_idx_hostaccess,
          linear: true,
        },
        vk::DeviceSize::max(minsize, 1 << 23),
      );
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
  pub fn get_image_memtype(&self, properties: vk::MemoryPropertyFlags, linear: bool) -> Option<Memtype> {
    Allocator::get_memtype(self.pdevice, &self.image_requirements, properties).map(|index| Memtype { index, linear })
  }

  /// Get the memtype of a buffer with the specified memory properties
  ///
  /// ## Returns
  ///  - An option with the memory type index.
  ///  - None, if there is no memory type that supports buffers with the specified properties
  pub fn get_buffer_memtype(&self, properties: vk::MemoryPropertyFlags) -> Option<Memtype> {
    Allocator::get_memtype(self.pdevice, &self.buffer_requirements, properties).map(|index| Memtype { index, linear: true })
  }

  /// Get the pagesize for the memory type
  ///
  /// # Returns
  ///  - The pagesize of the memory type, if one has been set in the AllocatorSizes.
  ///  - Otherwise the default pagesize is returned.
  pub fn get_pagesize(&self, memtype: Memtype) -> vk::DeviceSize {
    match self.pagesizes.get(&memtype) {
      Some(size) => *size,
      None => self.pagesize_default,
    }
  }

  /// Set the pagesize for the specifid memory type
  ///
  /// The page size must be at least of size `bufferImageGranularity`. If not the result returns [InvalidPageSize](enum.Error.html#field.InvalidPageSize).
  pub fn set_pagesize(&mut self, memtype: Memtype, size: vk::DeviceSize) -> Result<(), Error> {
    if size < Allocator::get_min_pagesize(self.pdevice) {
      Err(Error::InvalidPageSize)?
    }
    self.pagesizes.entry(memtype).or_insert(size);
    Ok(())
  }
}

struct AllocatorImpl {
  device: vk::Device,
  pagetbls: HashMap<Memtype, Table>,
  handles: HashMap<Handle<u64>, Memtype>,
}

impl Drop for AllocatorImpl {
  fn drop(&mut self) {
    for (h, _) in self.handles.iter() {
      match h {
        Handle::Buffer(h) => vk::DestroyBuffer(self.device, *h, std::ptr::null()),
        Handle::Image(h) => vk::DestroyImage(self.device, *h, std::ptr::null()),
      };
    }
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
/// Memory of pages that have no resource binding any more can be freed with [free_unused](struct.Allocator.html#method.free_unused) again.
///
/// ```rust
/// extern crate nobs_vk as vk;
/// extern crate nobs_vkmem as vkmem;
///
/// use vkmem::Handle;
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
/// // configure buffer/image create infos
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
///   // all BindTypes will NOT shuffle the order of the resources around
///   .bind(&mut allocator, vkmem::BindType::Scatter)
///   .unwrap();
///
/// // Mapped gives a convenient view on the memory
/// // get_mapped mapps the whole block of memory to which the resources was bound
/// // get_mapped_region lets us define a byte offset respective to the beginning of the resource and a size in bytes
/// {
///   let mapped = allocator.get_mapped(Handle::Buffer(buf_ub)).unwrap();
///   let ubb = Ub { a: 123, b: 4, c: 5 };
///   mapped.host_to_device(&ubb);
/// }
/// {
///   let mapped = allocator.get_mapped(Handle::Buffer(buf_ub)).unwrap();
///   let ubb : Ub = mapped.device_to_host();
/// }
///
/// {
///   let mapped = allocator.get_mapped_region(Handle::Buffer(buf_out), 4, 100).unwrap();
///   let v = mapped.as_slice::<u32>();
/// }
///
/// // we can print stats in a yaml format for the currently allocated pages
/// println!("{}", allocator.print_stats());
///
/// // buffers and images can be destroyed
/// allocator.destroy(Handle::Image(img));
/// allocator.destroy_many(&[Handle::Buffer(buf_ub), Handle::Buffer(buf_out)]);
///
/// // destroying does NOT free memory
/// // if we want to free memory we can do this only if a whole page is not used any more
/// // in this case we can free the memory again
/// //allocator.free_unused();
///
/// // dropping the allocator automatically destroys bound resources and frees all memory
/// # }
/// ```
#[derive(Clone)]
pub struct Allocator {
  device: vk::Device,
  sizes: std::sync::Arc<AllocatorSizes>,
  alloc: std::sync::Arc<std::sync::Mutex<AllocatorImpl>>,
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
    let mut properties = std::mem::MaybeUninit::uninit();
    vk::GetPhysicalDeviceProperties(pdevice, properties.as_mut_ptr());
    unsafe { properties.assume_init().limits.bufferImageGranularity }
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
    let mut device_properties = std::mem::MaybeUninit::uninit();
    vk::GetPhysicalDeviceMemoryProperties(pdevice, device_properties.as_mut_ptr());
    let device_properties = unsafe { device_properties.assume_init() };

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
      sizes: std::sync::Arc::new(sizes),
      alloc: std::sync::Arc::new(std::sync::Mutex::new(AllocatorImpl {
        device,
        pagetbls: Default::default(),
        handles: Default::default(),
      })),
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
  ///     vkmem::BindInfo::new(buf, host_access_flags, true),
  ///     vkmem::BindInfo::new(img, device_local_flags, false),
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
      let pageinfo = BindInfoInner::new(info, self.device);
      let memtype = Memtype {
        index: Self::get_memtype(self.sizes.pdevice, &pageinfo.requirements, info.properties).ok_or(Error::InvalidMemoryType)?,
        linear: info.linear,
      };
      by_memtype.entry(memtype).or_insert(Vec::new()).push(pageinfo);
    }

    let mut alloc = self.alloc.lock().unwrap();

    // for every group with the same memtype bind the buffers to a page table
    let device = self.device;
    for (memtype, infos) in by_memtype {
      let pagesize = self.sizes.get_pagesize(memtype);
      alloc
        .pagetbls
        .entry(memtype)
        .or_insert_with(|| Table::new(device, memtype, pagesize))
        .bind(&infos, bindtype)?;

      for h in infos.iter().map(|i| i.handle) {
        alloc.handles.insert(h, memtype);
      }
    }

    Ok(())
  }

  /// Destroys a resource, that has been bound to this allocator
  ///
  /// see [destroy_many](struct.Allocator.html#method.destroy_many)
  pub fn destroy(&mut self, handle: Handle<u64>) {
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
  /// # use vkmem::Handle;
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
  /// allocator.destroy_many(&[Handle::Buffer(buf), Handle::Image(img)]);
  /// # }
  /// ```
  pub fn destroy_many(&mut self, handles: &[Handle<u64>]) {
    Self::destroy_inner(self.device, &mut self.alloc.lock().unwrap(), handles);
  }

  fn destroy_inner(device: vk::Device, alloc: &mut AllocatorImpl, handles: &[Handle<u64>]) {
    let by_memtype = handles
      .iter()
      .filter_map(|h| alloc.handles.get(h).map(|mt| (*h, *mt)))
      .fold(HashMap::new(), |mut acc, (h, mt)| {
        acc.entry(mt).or_insert(Vec::new()).push(h);
        acc
      });

    for (mt, hs) in by_memtype.iter() {
      alloc.pagetbls.get_mut(mt).unwrap().unbind(hs);
    }

    for h in handles.iter() {
      if alloc.handles.remove(h).is_some() {
        match h {
          Handle::Buffer(h) => vk::DestroyBuffer(device, *h, std::ptr::null()),
          Handle::Image(h) => vk::DestroyImage(device, *h, std::ptr::null()),
        }
      }
    }
  }

  /// Frees memory of unused pages
  pub fn free_unused(&mut self) {
    for (_, tbl) in self.alloc.lock().unwrap().pagetbls.iter_mut() {
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

  fn get_mem(&self, handle: Handle<u64>) -> Option<Block> {
    let alloc = self.alloc.lock().unwrap();
    alloc
      .handles
      .get(&handle)
      .and_then(|t| alloc.pagetbls.get(&t))
      .and_then(|tbl| tbl.get_mem(handle))
  }

  /// Gets a [Mapped](mapped/struct.Mapped.html) of the spicified resource handle
  ///
  /// # Example
  /// Creates a uniform buffer, stores some values in it and reads them back
  ///```rust
  /// # extern crate nobs_vk as vk;
  /// # extern crate nobs_vkmem as vkmem;
  /// # use vkmem::Handle;
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
  ///   let mapped = allocator.get_mapped(Handle::Buffer(buf_ub)).unwrap();
  ///   let ubb = Ub { a: 123, b: 4, c: 5 };
  ///   mapped.host_to_device(&ubb);
  /// }
  /// {
  ///   let mapped = allocator.get_mapped(Handle::Buffer(buf_ub)).unwrap();
  ///   let ubb : Ub = mapped.device_to_host();
  ///   assert_eq!(ubb.a, 123);
  ///   assert_eq!(ubb.b, 4);
  ///   assert_eq!(ubb.c, 5);
  /// }
  /// # }
  /// ```
  pub fn get_mapped(&self, handle: Handle<u64>) -> Option<Mapped> {
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
  /// # use vkmem::Handle;
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
  ///   let mut mapped = allocator.get_mapped_region(Handle::Buffer(buf), 4, 100).unwrap();
  ///   let v = mapped.as_slice_mut::<u32>();
  ///   v[0] = 123;
  ///   v[1] = 4;
  ///   v[2] = 5;
  /// }
  /// {
  ///   let mapped = allocator.get_mapped_region(Handle::Buffer(buf), 4, 100).unwrap();
  ///   let v = mapped.as_slice::<u32>();
  ///   assert_eq!(v[0], 123);
  ///   assert_eq!(v[1], 4);
  ///   assert_eq!(v[2], 5);
  /// }
  /// # }
  /// ```
  pub fn get_mapped_region(&self, handle: Handle<u64>, offset: vk::DeviceSize, size: vk::DeviceSize) -> Option<Mapped> {
    self.get_mem(handle).and_then(|b| {
      let region = Block::new(b.mem, b.beg + b.pad + offset, b.beg + b.pad + offset + size, 0);
      match region.beg < b.end && region.end <= b.end {
        true => Mapped::new(self.device, region).ok(),
        false => None,
      }
    })
  }

  /// Print staticstics for the Allocator in yaml format
  pub fn print_stats(&self) -> String {
    let alloc = self.alloc.lock().unwrap();
    let mut s = String::new();

    let mut keys: Vec<_> = alloc.pagetbls.keys().collect();
    keys.sort();
    for k in keys {
      write!(s, "{}", alloc.pagetbls[k].print_stats()).unwrap();
    }
    s
  }
}
