use crate::Allocator;
use crate::BindInfo;
use crate::BindType;
use crate::Error;
use crate::Handle;
use vk;

/// Accumulator for buffer and image create infos
///
/// We accumulate create infos, so that we can bind them in bulk, which gives better memory utilization.
#[derive(Default)]
struct ResourceBuilder<'a> {
  handles: Vec<&'a mut u64>,

  infos: Vec<Handle<usize>>,
  buffers: Vec<BufferCreate>,
  images: Vec<ImageCreate>,
}

impl<'a> ResourceBuilder<'a> {
  /// Creates the buffers or images from their respective create infos
  ///
  /// Fails, if the vulkan command does not return successfully
  fn create_bindinfos(&self, device: vk::Device) -> Result<Vec<BindInfo>, Error> {
    let mut bindinfos = Vec::with_capacity(self.infos.len());

    // create buffers and handles from the infos
    for (i, info) in self.infos.iter().enumerate() {
      let bindinfo = match info {
        Handle::Buffer(j) => {
          let info = &self.buffers[*j];
          let mut h = vk::NULL_HANDLE;
          vk_check!(vk::CreateBuffer(device, &info.info, std::ptr::null(), &mut h)).map_err(|_| Error::CreateBufferFailed(i as u32))?;
          BindInfo::with_size(Handle::Buffer(h), info.info.size, info.properties)
        }
        Handle::Image(j) => {
          let info = &self.images[*j];
          let mut h = vk::NULL_HANDLE;
          vk_check!(vk::CreateImage(device, &info.info, std::ptr::null(), &mut h)).map_err(|_| Error::CreateImageFailed(i as u32))?;
          BindInfo::new(Handle::Image(h), info.properties)
        }
      };
      bindinfos.push(bindinfo);
    }
    Ok(bindinfos)
  }

  /// Deletes buffers/images in `bindinfos`
  ///
  /// This is needed in case resources had been created with [create_bindinfos](struct.ResourceBuilder.html#method.create_bindinfos)
  /// but binding the resources faild.
  fn delete_bindinfos(&self, device: vk::Device, bindinfos: &[BindInfo], e: Error) -> Error {
    for h in bindinfos.iter().map(|i| i.handle).filter(|h| h.get() != vk::NULL_HANDLE) {
      match h {
        Handle::Buffer(h) => vk::DestroyBuffer(device, h, std::ptr::null()),
        Handle::Image(h) => vk::DestroyImage(device, h, std::ptr::null()),
      }
    }
    e
  }

  /// Copies the locally created handles to their output reference
  fn copy_out_handles(&mut self, bindinfos: &[BindInfo]) -> Result<(), Error> {
    self
      .handles
      .iter_mut()
      .zip(bindinfos.iter())
      .for_each(|(out, info)| **out = info.handle.get());
    Ok(())
  }

  /// Create all accumulated buffers and images and bind them to the `allocator`
  fn bind(&mut self, allocator: &mut Allocator, bindtype: BindType) -> Result<(), Error> {
    let device = allocator.get_device();
    let bindinfos = self.create_bindinfos(device)?;
    allocator
      .bind(&bindinfos, bindtype)
      .or_else(|e| Err(self.delete_bindinfos(device, &bindinfos, e)))?;
    self.copy_out_handles(&bindinfos)
  }

  /// Add the `buffer` the the accumulator
  ///
  /// The buffer will be created when bind is called
  fn add_buffer(&mut self, handle: &'a mut u64, buffer: BufferCreate) {
    self.handles.push(handle);
    self.infos.push(Handle::Buffer(self.buffers.len()));
    self.buffers.push(buffer);
  }

  /// Add the `image` the the accumulator
  ///
  /// The image will be created when bind is called
  fn add_image(&mut self, handle: &'a mut u64, image: ImageCreate) {
    self.handles.push(handle);
    self.infos.push(Handle::Image(self.images.len()));
    self.images.push(image);
  }
}

/// Generic builder for buffers and images
///
/// This builder can neither configure a buffer nor an image, but is useful to have when resources are created in a loop.
///
/// ## Exapmle
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
/// let mut handles = vec![vk::NULL_HANDLE, vk::NULL_HANDLE, vk::NULL_HANDLE];
/// let mut builder = vkmem::Resource::new();
/// for h in handles.iter_mut() {
///   builder = builder
///     .new_buffer(h)
///     .size(123)
///     .devicelocal(true)
///     .usage(vk::BUFFER_USAGE_TRANSFER_DST_BIT | vk::BUFFER_USAGE_STORAGE_BUFFER_BIT)
///     .submit()
/// }
/// builder.bind(&mut allocator, vkmem::BindType::Scatter);
///
/// assert!(handles[0] != vk::NULL_HANDLE);
/// assert!(handles[1] != vk::NULL_HANDLE);
/// assert!(handles[2] != vk::NULL_HANDLE);
/// # }
/// ```
pub struct Resource<'a> {
  builder: ResourceBuilder<'a>,
}

impl<'a> Resource<'a> {
  pub fn new() -> Self {
    Self {
      builder: Default::default(),
    }
  }

  fn with_builder(builder: ResourceBuilder<'a>) -> Self {
    Self { builder }
  }

  /// Starts configuration of a new buffer resource
  ///
  /// The new builder will be initialized as in [new](struct.Buffer.html#method.new)
  pub fn new_buffer(self, handle: &'a mut u64) -> Buffer {
    Buffer::with_builder(handle, self.builder)
  }

  /// Starts configuration of a new image resource
  ///
  /// The new builder will be initialized as in [new](struct.Image.html#method.new)
  pub fn new_image(self, handle: &'a mut u64) -> Image {
    Image::with_builder(handle, self.builder)
  }

  /// Creates resources and binds them to the specified allocator
  pub fn bind(mut self, alloc: &mut Allocator, ty: BindType) -> Result<(), Error> {
    self.builder.bind(alloc, ty)
  }
}

/// Buffer create info plus memory properties
struct BufferCreate {
  family_indices: Vec<u32>,
  info: vk::BufferCreateInfo,
  properties: vk::MemoryPropertyFlags,
}

/// Builder pattern for creating buffer resources
///
/// A new builder will be initialized with the [default](struct.Buffer.html#method.new) configuration.
///
/// After the buffer has been configured, it can be created and bound to an allocator with [bind](struct.Buffer.html#method.bind).
///
/// One can create multiple buffers/images in bulk by calling [next_buffer](struct.Buffer.html#method.next_buffer) or [next_image](struct.Buffer.html#method.next_image).
/// This has the benefit, that then they are more likely to share a commen momory block.
///
/// see [Allocator](struct.Allocator.html) for more details on creating and binding buffers.
pub struct Buffer<'a> {
  builder: ResourceBuilder<'a>,
  handle: &'a mut u64,
  buffer: BufferCreate,
}

impl<'a> Buffer<'a> {
  /// Creates a new builder.
  ///
  /// By default the builder will be initialized with
  ///  - memory properties: `vk::MEMORY_PROPERTY_DEVICE_LOCAL_BIT`
  ///  - size: 0
  ///  - usage: 0
  ///  - sharingMode: `vk::SHARING_MODE_EXCLUSIVE` (no queue indices)
  ///
  /// After [bind](struct.Buffer.html#method.bind) is called, the created buffer will be copied into the specified `handle`
  pub fn new(handle: &'a mut u64) -> Self {
    Self::with_builder(handle, Default::default())
  }

  fn with_builder(handle: &'a mut u64, builder: ResourceBuilder<'a>) -> Self {
    Self {
      builder,
      handle,
      buffer: BufferCreate {
        family_indices: Default::default(),
        info: vk::BufferCreateInfo {
          sType: vk::STRUCTURE_TYPE_BUFFER_CREATE_INFO,
          pNext: std::ptr::null(),
          flags: 0,
          size: 0,
          usage: 0,
          sharingMode: vk::SHARING_MODE_EXCLUSIVE,
          queueFamilyIndexCount: 0,
          pQueueFamilyIndices: std::ptr::null(),
        },
        properties: vk::MEMORY_PROPERTY_DEVICE_LOCAL_BIT,
      },
    }
  }

  /// Sets the buffer size in bytes
  pub fn size(mut self, size: vk::DeviceSize) -> Self {
    self.buffer.info.size = size;
    self
  }

  /// Sets the buffer usage
  pub fn usage(mut self, usage: vk::BufferUsageFlags) -> Self {
    self.buffer.info.usage = usage;
    self
  }

  /// Sets the buffer sharing mode
  pub fn sharing(mut self, sharing: vk::SharingMode) -> Self {
    self.buffer.info.sharingMode = sharing;
    self
  }

  /// Sets the buffers queue indices for sharing
  ///
  /// This will be ignored if the sharing mode is `vk::SHARING_MODE_EXCLUSIVE`
  pub fn queues(mut self, queue_family_indices: &[u32]) -> Self {
    self.buffer.family_indices = queue_family_indices.to_vec();
    self.buffer.info.queueFamilyIndexCount = self.buffer.family_indices.len() as u32;
    self.buffer.info.pQueueFamilyIndices = self.buffer.family_indices.as_ptr();
    self
  }

  /// Sets the memory properties of the buffer
  pub fn mem_properties(mut self, properties: vk::MemoryPropertyFlags) -> Self {
    self.buffer.properties = properties;
    self
  }

  /// Setst the memory properties of the buffer
  ///
  /// Sets the properties to `vk::MEMORY_PROPERTY_DEVICE_LOCAL_BIT` if `local` is true.
  /// Other wise sets properties to `vk::MEMORY_PROPERTY_HOST_VISIBLE_BIT | vk::MEMORY_PROPERTY_HOST_COHERENT_BIT`.
  pub fn devicelocal(self, local: bool) -> Self {
    if local {
      self.mem_properties(vk::MEMORY_PROPERTY_DEVICE_LOCAL_BIT)
    } else {
      self.mem_properties(vk::MEMORY_PROPERTY_HOST_VISIBLE_BIT | vk::MEMORY_PROPERTY_HOST_COHERENT_BIT)
    }
  }

  /// Finishes configuration of this buffer
  ///
  /// ## Returns
  /// A [Resource](struct.Resource.html) so that we can continue configuring new buffers/images.
  pub fn submit(mut self) -> Resource<'a> {
    self.builder.add_buffer(self.handle, self.buffer);
    Resource::with_builder(self.builder)
  }

  /// Short hand for [`submit()`](struct.Image.html#method.submit).[`new_buffer(handle)`](struct.Resource.html#method.new_buffer)
  pub fn new_buffer(self, handle: &'a mut u64) -> Self {
    self.submit().new_buffer(handle)
  }

  /// Short hand for [`submit()`](struct.Image.html#method.submit).[`new_image(handle)`](struct.Resource.html#method.new_image)
  pub fn new_image(self, handle: &'a mut u64) -> Image {
    self.submit().new_image(handle)
  }

  /// Short hand for [`submit()`](struct.Image.html#method.submit).[`bind(handle)`](struct.Resource.html#method.bind)
  pub fn bind(self, allocator: &mut Allocator, bindtype: BindType) -> Result<(), Error> {
    self.submit().bind(allocator, bindtype)
  }
}

/// Image create info plus memory properties
struct ImageCreate {
  family_indices: Vec<u32>,
  info: vk::ImageCreateInfo,
  properties: vk::MemoryPropertyFlags,
}

/// Builder pattern for creating image resources
///
/// A new builder will be initialized with the [default](struct.Image.html#method.new) configuration.
///
/// After the image has been configured, it can be created and bound to an allocator with [bind](struct.Image.html#method.bind).
///
/// One can create multiple buffers/images in bulk by calling [next_buffer](struct.Image.html#method.next_buffer) or [next_image](struct.Image.html#method.next_image).
/// This has the benefit, that then they are more likely to share a commen momory block.
///
/// see [Allocator](struct.Allocator.html) for more details on creating and binding images.
pub struct Image<'a> {
  builder: ResourceBuilder<'a>,
  handle: &'a mut u64,
  image: ImageCreate,
}

impl<'a> Image<'a> {
  /// Creates a new builder.
  ///
  /// Initializes with [defaults](struct.Image.html#method.defaults)
  ///
  /// After [bind](struct.Image.html#method.bind) is called, the created image will be copied into the specified `handle`
  pub fn new(handle: &'a mut u64) -> Self {
    Self::with_builder(handle, Default::default())
  }

  fn with_builder(handle: &'a mut u64, builder: ResourceBuilder<'a>) -> Self {
    Self {
      builder,
      handle,
      image: ImageCreate {
        family_indices: Default::default(),
        info: unsafe { std::mem::uninitialized() },
        properties: 0,
      },
    }
    .defaults()
  }

  /// Sets the default image configuration
  ///
  /// By default the builder will be initialized with
  ///  - memory properties: `vk::MEMORY_PROPERTY_DEVICE_LOCAL_BIT`
  ///  - imageType: `vk::IMAGE_TYPE_2D`
  ///  - format: `vk::FORMAT_R8G8B8A8_UNORM`
  ///  - extent: width = height = depth = 0
  ///  - mipLevels: 1
  ///  - arrayLayers: 1
  ///  - samples: `vk::SAMPLE_COUNT_1_BIT`
  ///  - tiling: `vk::IMAGE_TILING_OPTIMAL`
  ///  - usage: 0,
  ///  - initialLayout: `vk::IMAGE_LAYOUT_UNDEFINED`
  ///  - sharingMode: `vk::SHARING_MODE_EXCLUSIVE (no queue indices)
  pub fn defaults(mut self) -> Self {
    self.image.info.sType = vk::STRUCTURE_TYPE_IMAGE_CREATE_INFO;
    self.image.info.pNext = std::ptr::null();
    self.image.info.flags = 0;
    self
      .image_type(vk::IMAGE_TYPE_2D)
      .format(vk::FORMAT_B8G8R8A8_UNORM)
      .size(1, 1, 1)
      .mip_levels(1)
      .array_layers(1)
      .samples(vk::SAMPLE_COUNT_1_BIT)
      .tiling(vk::IMAGE_TILING_OPTIMAL)
      .usage(0)
      .sharing(vk::SHARING_MODE_EXCLUSIVE)
      .queues(&[])
      .layout(vk::IMAGE_LAYOUT_UNDEFINED)
      .devicelocal(true)
  }

  /// Sets the configuration to be used as a sampled 2D texture
  ///
  /// Basically sets the defaults with:
  ///  - width: `w`
  ///  - height: `h`
  ///  - format: `format`.
  ///  - usage: `vk::IMAGE_USAGE_TRANSFER_SRC_BIT | vk::IMAGE_USAGE_TRANSFER_DST_BIT | vk::IMAGE_USAGE_SAMPLED_BIT`
  pub fn texture2d(self, w: u32, h: u32, format: vk::Format) -> Self {
    self
      .defaults()
      .format(format)
      .width(w)
      .height(h)
      .usage(vk::IMAGE_USAGE_TRANSFER_SRC_BIT | vk::IMAGE_USAGE_TRANSFER_DST_BIT | vk::IMAGE_USAGE_SAMPLED_BIT)
  }

  /// Sets the configuration to be used as a color attachment
  ///
  /// This is basically a [texture2D](struct.Image.html#method.texture2D) with additional usage `vk::IMAGE_USAGE_COLOR_ATTACHMENT_BIT`
  pub fn color_attachment(self, w: u32, h: u32, format: vk::Format) -> Self {
    self.texture2d(w, h, format).usage(
      vk::IMAGE_USAGE_TRANSFER_SRC_BIT
        | vk::IMAGE_USAGE_TRANSFER_DST_BIT
        | vk::IMAGE_USAGE_SAMPLED_BIT
        | vk::IMAGE_USAGE_COLOR_ATTACHMENT_BIT,
    )
  }

  /// Sets the configuration to be used as a depth attachment
  ///
  /// This is basically a [texture2D](struct.Image.html#method.texture2D) with usage `vk::IMAGE_USAGE_DEPTH_STENCIL_ATTACHMENT_BIT`
  pub fn depth_attachment(self, w: u32, h: u32, format: vk::Format) -> Self {
    self.texture2d(w, h, format).usage(vk::IMAGE_USAGE_DEPTH_STENCIL_ATTACHMENT_BIT)
  }

  /// Set the image type
  pub fn image_type(mut self, ty: vk::ImageType) -> Self {
    self.image.info.imageType = ty;
    self
  }

  /// Set the image format
  pub fn format(mut self, format: vk::Format) -> Self {
    self.image.info.format = format;
    self
  }

  /// Set the width in pixels
  pub fn width(mut self, w: u32) -> Self {
    self.image.info.extent.width = w;
    self
  }
  /// Set the height in pixels
  pub fn height(mut self, h: u32) -> Self {
    self.image.info.extent.height = h;
    self
  }
  /// Set the depth in pixels
  pub fn depth(mut self, d: u32) -> Self {
    self.image.info.extent.depth = d;
    self
  }
  /// Set the width, height and depth in pixel
  pub fn size(mut self, w: u32, h: u32, d: u32) -> Self {
    self.image.info.extent.width = w;
    self.image.info.extent.height = h;
    self.image.info.extent.depth = d;
    self
  }
  /// Set the width, height and depth in pixel
  pub fn extent(mut self, extent: vk::Extent3D) -> Self {
    self.image.info.extent = extent;
    self
  }

  /// Set the number of mip level
  pub fn mip_levels(mut self, levels: u32) -> Self {
    self.image.info.mipLevels = levels;
    self
  }

  /// Set the number of array layers
  pub fn array_layers(mut self, layers: u32) -> Self {
    self.image.info.arrayLayers = layers;
    self
  }

  /// Seth the multisampling properties
  pub fn samples(mut self, samples: vk::SampleCountFlags) -> Self {
    self.image.info.samples = samples;
    self
  }

  /// Set the image tiling
  pub fn tiling(mut self, tiling: vk::ImageTiling) -> Self {
    self.image.info.tiling = tiling;
    self
  }

  /// set the usage of the image
  pub fn usage(mut self, usage: vk::BufferUsageFlags) -> Self {
    self.image.info.usage = usage;
    self
  }

  /// Sets the image's sharing mode
  pub fn sharing(mut self, sharing: vk::SharingMode) -> Self {
    self.image.info.sharingMode = sharing;
    self
  }

  /// Sets the images queue indices for sharing
  ///
  /// This will be ignored if the sharing mode is `vk::SHARING_MODE_EXCLUSIVE`
  pub fn queues(mut self, queue_family_indices: &[u32]) -> Self {
    self.image.family_indices = queue_family_indices.to_vec();
    self.image.info.queueFamilyIndexCount = self.image.family_indices.len() as u32;
    self.image.info.pQueueFamilyIndices = self.image.family_indices.as_ptr();
    self
  }

  /// Sets the initial layout of the image
  pub fn layout(mut self, layout: vk::ImageLayout) -> Self {
    self.image.info.initialLayout = layout;
    self
  }

  /// Sets the memory properties of the buffer
  pub fn mem_properties(mut self, properties: vk::MemoryPropertyFlags) -> Self {
    self.image.properties = properties;
    self
  }

  /// Setst the memory properties of the image
  ///
  /// Sets the properties to `vk::MEMORY_PROPERTY_DEVICE_LOCAL_BIT` if `local` is true.
  /// Other wise sets properties to `vk::MEMORY_PROPERTY_HOST_VISIBLE_BIT | vk::MEMORY_PROPERTY_HOST_COHERENT_BIT`.
  pub fn devicelocal(self, local: bool) -> Self {
    if local {
      self.mem_properties(vk::MEMORY_PROPERTY_DEVICE_LOCAL_BIT)
    } else {
      self.mem_properties(vk::MEMORY_PROPERTY_HOST_VISIBLE_BIT | vk::MEMORY_PROPERTY_HOST_COHERENT_BIT)
    }
  }

  /// Finishes configuration of this image
  ///
  /// ## Returns
  /// A [Resource](struct.Resource.html) so that we can continue configuring new buffers/images.
  pub fn submit(mut self) -> Resource<'a> {
    self.builder.add_image(self.handle, self.image);
    Resource::with_builder(self.builder)
  }

  /// Short hand for [`submit()`](struct.Image.html#method.submit).[`new_buffer(handle)`](struct.Resource.html#method.new_buffer)
  pub fn new_buffer(self, handle: &'a mut u64) -> Buffer {
    self.submit().new_buffer(handle)
  }

  /// Short hand for [`submit()`](struct.Image.html#method.submit).[`new_image(handle)`](struct.Resource.html#method.new_image)
  pub fn new_image(self, handle: &'a mut u64) -> Self {
    self.submit().new_image(handle)
  }

  /// Short hand for [`submit()`](struct.Image.html#method.submit).[`bind(handle)`](struct.Resource.html#method.bind)
  pub fn bind(self, allocator: &mut Allocator, bindtype: BindType) -> Result<(), Error> {
    self.submit().bind(allocator, bindtype)
  }
}

pub struct ImageView {
  device: vk::Device,
  info: vk::ImageViewCreateInfo,
}

impl ImageView {
  pub fn new(device: vk::Device, image: vk::Image) -> Self {
    Self {
      device,
      info: vk::ImageViewCreateInfo {
        sType: vk::STRUCTURE_TYPE_IMAGE_VIEW_CREATE_INFO,
        pNext: std::ptr::null(),
        flags: 0,
        image: image,
        viewType: vk::IMAGE_VIEW_TYPE_2D,
        format: vk::FORMAT_UNDEFINED,
        components: vk::ComponentMapping {
          r: vk::COMPONENT_SWIZZLE_IDENTITY,
          g: vk::COMPONENT_SWIZZLE_IDENTITY,
          b: vk::COMPONENT_SWIZZLE_IDENTITY,
          a: vk::COMPONENT_SWIZZLE_IDENTITY,
        },
        subresourceRange: vk::ImageSubresourceRange {
          aspectMask: 0,
          baseMipLevel: 0,
          levelCount: 1,
          baseArrayLayer: 0,
          layerCount: 1,
        },
      },
    }
  }

  pub fn view_type(mut self, ty: vk::ImageViewType) -> Self {
    self.info.viewType = ty;
    self
  }

  pub fn format(mut self, format: vk::Format) -> Self {
    self.info.format = format;
    self
  }

  pub fn compontents(mut self, components: vk::ComponentMapping) -> Self {
    self.info.components = components;
    self
  }

  pub fn subresource(mut self, subresource: vk::ImageSubresourceRange) -> Self {
    self.info.subresourceRange = subresource;
    self
  }
  pub fn aspect(mut self, aspect: vk::ImageAspectFlags) -> Self {
    self.info.subresourceRange.aspectMask = aspect;
    self
  }

  pub fn create(&self) -> Result<vk::ImageView, vk::Error> {
    let mut view = vk::NULL_HANDLE;
    vk_check!(vk::CreateImageView(self.device, &self.info, std::ptr::null(), &mut view))?;
    Ok(view)
  }
}
