use vk;

/// Builder for a descriptor buffer info to write a buffer binding
/// 
/// Default initialized to vk::NULL_HANDLE as buffer with offset 0 and range vk::WHOLE_SIZE
pub struct DescriptorBufferInfoBuilder {
  info: vk::DescriptorBufferInfo,
}

impl DescriptorBufferInfoBuilder {
  pub fn set(mut self, buffer: vk::Buffer, offset: vk::DeviceSize, range: vk::DeviceSize) -> DescriptorBufferInfoBuilder {
    self.info = vk::DescriptorBufferInfo { buffer, offset, range };
    self
  }

  pub fn buffer(mut self, buffer: vk::Buffer) -> DescriptorBufferInfoBuilder {
    self.info.buffer = buffer;
    self
  }

  pub fn offset(mut self, offset: vk::DeviceSize) -> DescriptorBufferInfoBuilder {
    self.info.offset = offset;
    self
  }

  pub fn range(mut self, range: vk::DeviceSize) -> DescriptorBufferInfoBuilder {
    self.info.range = range;
    self
  }

  pub fn get(self) -> vk::DescriptorBufferInfo {
    self.info
  }
}

impl Default for DescriptorBufferInfoBuilder {
  fn default() -> DescriptorBufferInfoBuilder {
    DescriptorBufferInfoBuilder {
      info: vk::DescriptorBufferInfo {
        buffer: vk::NULL_HANDLE,
        offset: 0,
        range: vk::WHOLE_SIZE,
      },
    }
  }
}

/// Builder for a descriptor image info to write an image binding
/// 
/// Default initialized to vk::NULL_HANDLE as image view and sampler and vk::IMAGE_LAYOUT_UNDEFINED as image layout
pub struct DescriptorImageInfoBuilder {
  info: vk::DescriptorImageInfo,
}

impl DescriptorImageInfoBuilder {
  /// Create builder with the specified image layout and vk::NULL_HANDLEs for image view and sampler
  pub fn with_layout(layout: vk::ImageLayout) -> DescriptorImageInfoBuilder {
    DescriptorImageInfoBuilder {
      info: vk::DescriptorImageInfo {
        imageLayout: layout,
        imageView: vk::NULL_HANDLE,
        sampler: vk::NULL_HANDLE,
      },
    }
  }

  pub fn set(mut self, layout: vk::ImageLayout, image_view: vk::ImageView, sampler: vk::Sampler) -> DescriptorImageInfoBuilder {
    self.info = vk::DescriptorImageInfo {
      imageLayout: layout,
      imageView: image_view,
      sampler,
    };
    self
  }

  pub fn layout(mut self, layout: vk::ImageLayout) -> DescriptorImageInfoBuilder {
    self.info.imageLayout = layout;
    self
  }

  pub fn image(mut self, view: vk::ImageView) -> DescriptorImageInfoBuilder {
    self.info.imageView = view;
    self
  }

  pub fn sampler(mut self, sampler: vk::Sampler) -> DescriptorImageInfoBuilder {
    self.info.sampler = sampler;
    self
  }

  pub fn get(self) -> vk::DescriptorImageInfo {
    self.info
  }
}

impl Default for DescriptorImageInfoBuilder {
  fn default() -> DescriptorImageInfoBuilder {
    Self::with_layout(vk::IMAGE_LAYOUT_UNDEFINED)
  }
}

pub enum WriteOffset {
  Buffer(isize),
  Image(isize),
  BufferView(isize),
}

/// Builder for updating a descriptor set
pub struct Writes<'a> {
  device: &'a vk::DeviceExtensions,
  dset: vk::DescriptorSet,
  writes: Vec<vk::WriteDescriptorSet>,
  write_offsets: Vec<WriteOffset>,
  buffer_infos: Vec<vk::DescriptorBufferInfo>,
  image_infos: Vec<vk::DescriptorImageInfo>,
  buffer_views: Vec<vk::BufferView>,
}

impl<'a> Writes<'a> {
  /// Creates a new builder for the specified descriptor set and device with no writes
  pub fn new(device: &'a vk::DeviceExtensions, dset: vk::DescriptorSet) -> Writes {
    Writes::<'a> {
      device,
      dset,
      writes: Default::default(),
      write_offsets: Default::default(),
      buffer_infos: Default::default(),
      image_infos: Default::default(),
      buffer_views: Default::default(),
    }
  }

  /// Sets the sepecified buffer for the binding, array element and descriptor type
  pub fn push_buffer(&mut self, binding: u32, array_elem: u32, ty: vk::DescriptorType, info: vk::DescriptorBufferInfo) {
    self.writes.push(vk::WriteDescriptorSet {
      sType: vk::STRUCTURE_TYPE_WRITE_DESCRIPTOR_SET,
      pNext: std::ptr::null(),
      dstSet: self.dset,
      dstBinding: binding,
      dstArrayElement: array_elem,
      descriptorType: ty,
      descriptorCount: 1,
      pBufferInfo: std::ptr::null(),
      pImageInfo: std::ptr::null(),
      pTexelBufferView: std::ptr::null(),
    });

    debug_assert!(
      ty & (vk::DESCRIPTOR_TYPE_UNIFORM_BUFFER
        | vk::DESCRIPTOR_TYPE_STORAGE_BUFFER
        | vk::DESCRIPTOR_TYPE_UNIFORM_BUFFER_DYNAMIC
        | vk::DESCRIPTOR_TYPE_STORAGE_BUFFER_DYNAMIC)
        != 0
    );

    self.write_offsets.push(WriteOffset::Buffer(self.buffer_infos.len() as isize));
    self.buffer_infos.push(info);
  }

  /// Sets the sepecified image for the binding, array element and descriptor type
  pub fn push_image(&mut self, binding: u32, array_elem: u32, ty: vk::DescriptorType, info: vk::DescriptorImageInfo) {
    self.writes.push(vk::WriteDescriptorSet {
      sType: vk::STRUCTURE_TYPE_WRITE_DESCRIPTOR_SET,
      pNext: std::ptr::null(),
      dstSet: self.dset,
      dstBinding: binding,
      dstArrayElement: array_elem,
      descriptorType: ty,
      descriptorCount: 1,
      pBufferInfo: std::ptr::null(),
      pImageInfo: std::ptr::null(),
      pTexelBufferView: std::ptr::null(),
    });

    debug_assert!(
      ty & (vk::DESCRIPTOR_TYPE_SAMPLER
        | vk::DESCRIPTOR_TYPE_COMBINED_IMAGE_SAMPLER
        | vk::DESCRIPTOR_TYPE_SAMPLED_IMAGE
        | vk::DESCRIPTOR_TYPE_STORAGE_IMAGE)
        != 0
    );

    self.write_offsets.push(WriteOffset::Buffer(self.image_infos.len() as isize));
    self.image_infos.push(info);
  }

  /// Sets the sepecified buffer view for the binding, array element and descriptor type
  pub fn push_bufferview(&mut self, binding: u32, array_elem: u32, ty: vk::DescriptorType, info: vk::BufferView) {
    self.writes.push(vk::WriteDescriptorSet {
      sType: vk::STRUCTURE_TYPE_WRITE_DESCRIPTOR_SET,
      pNext: std::ptr::null(),
      dstSet: self.dset,
      dstBinding: binding,
      dstArrayElement: array_elem,
      descriptorType: ty,
      descriptorCount: 1,
      pBufferInfo: std::ptr::null(),
      pImageInfo: std::ptr::null(),
      pTexelBufferView: std::ptr::null(),
    });

    debug_assert!(ty & (vk::DESCRIPTOR_TYPE_UNIFORM_TEXEL_BUFFER | vk::DESCRIPTOR_TYPE_STORAGE_TEXEL_BUFFER) != 0);

    self.write_offsets.push(WriteOffset::Buffer(self.buffer_views.len() as isize));
    self.buffer_views.push(info);
  }

  /// Adds a new buffer write for the specified binding and array with the builder returned by the lambda
  pub fn buffer<F: Fn(DescriptorBufferInfoBuilder) -> DescriptorBufferInfoBuilder>(
    &'a mut self,
    binding: u32,
    array_elem: u32,
    ty: vk::DescriptorType,
    f: F,
  ) -> &'a mut Writes {
    self.push_buffer(binding, array_elem, ty, f(DescriptorBufferInfoBuilder::default()).get());
    self
  }

  /// Adds a new image write for the specified binding and array with the builder returned by the lambda
  pub fn image<F: Fn(DescriptorImageInfoBuilder) -> DescriptorImageInfoBuilder>(
    &'a mut self,
    binding: u32,
    array_elem: u32,
    ty: vk::DescriptorType,
    f: F,
  ) -> &'a mut Writes {
    self.push_image(binding, array_elem, ty, f(DescriptorImageInfoBuilder::default()).get());
    self
  }

  /// Adds a new buffer view write for the specified binding and array
  pub fn buffer_view(&'a mut self, binding: u32, array_elem: u32, ty: vk::DescriptorType, view: vk::BufferView) -> &'a mut Writes {
    self.push_bufferview(binding, array_elem, ty, view);
    self
  }

  /// Updates the descriptor set with the configured writes
  pub fn update(&mut self) {
    for i in 0..self.writes.len() {
      match self.write_offsets[i] {
        WriteOffset::Buffer(off) => self.writes[i].pBufferInfo = unsafe { self.buffer_infos.as_ptr().offset(off) },
        WriteOffset::Image(off) => self.writes[i].pImageInfo = unsafe { self.image_infos.as_ptr().offset(off) },
        WriteOffset::BufferView(off) => self.writes[i].pTexelBufferView = unsafe { self.buffer_views.as_ptr().offset(off) },
      }
    }

    vk::UpdateDescriptorSets(
      self.device.get_handle(),
      self.writes.len() as u32,
      self.writes.as_ptr(),
      0,
      std::ptr::null(),
    );
  }
}
