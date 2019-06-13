use vk;

/// Builder for a descriptor buffer info to write a buffer binding
///
/// Default initialized to
///  - buffer: vk::NULL_HANDLE
///  - offset: 0
///  - range: vk::WHOLE_SIZE
pub struct DescriptorBufferInfoBuilder {
  info: vk::DescriptorBufferInfo,
}

impl DescriptorBufferInfoBuilder {
  /// Sets the complete configuration
  ///
  /// ## Arguments
  /// * `buffer` - buffer handle to write
  /// * `offset` - offset into the buffer in bytes
  /// * `range` - size of the bound range of the buffer
  pub fn set(mut self, buffer: vk::Buffer, offset: vk::DeviceSize, range: vk::DeviceSize) -> DescriptorBufferInfoBuilder {
    self.info = vk::DescriptorBufferInfo { buffer, offset, range };
    self
  }

  /// Sets the buffer handle
  pub fn buffer(mut self, buffer: vk::Buffer) -> DescriptorBufferInfoBuilder {
    self.info.buffer = buffer;
    self
  }

  /// Sets the off set into the buffer
  pub fn offset(mut self, offset: vk::DeviceSize) -> DescriptorBufferInfoBuilder {
    self.info.offset = offset;
    self
  }

  /// Sets the bound size of the buffer
  pub fn range(mut self, range: vk::DeviceSize) -> DescriptorBufferInfoBuilder {
    self.info.range = range;
    self
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

vk_builder_into!(vk::DescriptorBufferInfo, DescriptorBufferInfoBuilder);

/// Builder for a descriptor image info to write an image binding
///
/// Default initialized to
///  - image view: `vk::NULL_HANDLE`
///  - sampler: `vk::NULL_HANDLE`
///  - layout: `vk::IMAGE_LAYOUT_UNDEFINED`
pub struct DescriptorImageInfoBuilder {
  info: vk::DescriptorImageInfo,
}

impl DescriptorImageInfoBuilder {
  /// Create builder with the specified image layout and `vk::NULL_HANDLE`s for image view and sampler
  pub fn with_layout(layout: vk::ImageLayout) -> DescriptorImageInfoBuilder {
    DescriptorImageInfoBuilder {
      info: vk::DescriptorImageInfo {
        imageLayout: layout,
        imageView: vk::NULL_HANDLE,
        sampler: vk::NULL_HANDLE,
      },
    }
  }

  /// Sets the complete configuration
  ///
  /// ## Arguments
  /// * `layout` - layout of the image
  /// * `image_view` - image view to be bound
  /// * `sampler` - sampler used for the image
  pub fn set(mut self, layout: vk::ImageLayout, image_view: vk::ImageView, sampler: vk::Sampler) -> DescriptorImageInfoBuilder {
    self.info = vk::DescriptorImageInfo {
      imageLayout: layout,
      imageView: image_view,
      sampler,
    };
    self
  }

  /// Sets the layout of the bound image view
  pub fn layout(mut self, layout: vk::ImageLayout) -> DescriptorImageInfoBuilder {
    self.info.imageLayout = layout;
    self
  }

  /// Sets the image view to be bound
  pub fn image(mut self, view: vk::ImageView) -> DescriptorImageInfoBuilder {
    self.info.imageView = view;
    self
  }

  /// Sets the sampler to be bound
  pub fn sampler(mut self, sampler: vk::Sampler) -> DescriptorImageInfoBuilder {
    self.info.sampler = sampler;
    self
  }
}

impl Default for DescriptorImageInfoBuilder {
  fn default() -> DescriptorImageInfoBuilder {
    Self::with_layout(vk::IMAGE_LAYOUT_UNDEFINED)
  }
}

vk_builder_into!(vk::DescriptorImageInfo, DescriptorImageInfoBuilder);

enum WriteOffset {
  Buffer(isize),
  Image(isize),
  BufferView(isize),
}

/// Builder for updating a descriptor set
///
/// Aggregates `vk::DescriptorBufferInfo`s, `vk::DescriptorImageInfo`s and `vk::BufferView`s that have been configured with [buffer](struct.Writes.html#method.buffer)
/// or [image](struct.Writes.html#method.image) or [buffer_view](struct.Writes.html#method.buffer_view).
pub struct Writes {
  device: vk::Device,
  dset: vk::DescriptorSet,
  writes: Vec<vk::WriteDescriptorSet>,
  write_offsets: Vec<WriteOffset>,
  buffer_infos: Vec<vk::DescriptorBufferInfo>,
  image_infos: Vec<vk::DescriptorImageInfo>,
  buffer_views: Vec<vk::BufferView>,
}

impl Writes {
  /// Creates a new builder for the specified descriptor set and device with no writes
  ///
  /// ## Arguments
  /// * `device` - device handle
  /// * `dset` - descriptor set to be updated
  pub fn new(device: vk::Device, dset: vk::DescriptorSet) -> Writes {
    Writes {
      device,
      dset,
      writes: Default::default(),
      write_offsets: Default::default(),
      buffer_infos: Default::default(),
      image_infos: Default::default(),
      buffer_views: Default::default(),
    }
  }

  /// Sets the sepecified buffer for the binding and derscriptor type, at array element
  pub fn buffer(&mut self, binding: u32, array_elem: u32, ty: vk::DescriptorType, info: vk::DescriptorBufferInfo) {
    self.buffers(binding, array_elem, ty, &[info])
  }

  /// Sets the sepecified buffers for the binding and descriptor type, starting at array element
  pub fn buffers(&mut self, binding: u32, array_elem: u32, ty: vk::DescriptorType, infos: &[vk::DescriptorBufferInfo]) {
    self.writes.push(vk::WriteDescriptorSet {
      sType: vk::STRUCTURE_TYPE_WRITE_DESCRIPTOR_SET,
      pNext: std::ptr::null(),
      dstSet: self.dset,
      dstBinding: binding,
      dstArrayElement: array_elem,
      descriptorType: ty,
      descriptorCount: infos.len() as u32,
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
    for info in infos.iter() {
      self.buffer_infos.push(*info);
    }
  }

  /// Sets the sepecified image for the binding and derscriptor type, at array element
  pub fn image(&mut self, binding: u32, array_elem: u32, ty: vk::DescriptorType, info: vk::DescriptorImageInfo) {
    self.images(binding, array_elem, ty, &[info]);
  }

  /// Sets the sepecified image for the binding and descriptor type, starting at array element
  pub fn images(&mut self, binding: u32, array_elem: u32, ty: vk::DescriptorType, infos: &[vk::DescriptorImageInfo]) {
    self.writes.push(vk::WriteDescriptorSet {
      sType: vk::STRUCTURE_TYPE_WRITE_DESCRIPTOR_SET,
      pNext: std::ptr::null(),
      dstSet: self.dset,
      dstBinding: binding,
      dstArrayElement: array_elem,
      descriptorType: ty,
      descriptorCount: infos.len() as u32,
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

    self.write_offsets.push(WriteOffset::Image(self.image_infos.len() as isize));
    for info in infos.iter() {
      self.image_infos.push(*info);
    }
  }

  /// Sets the sepecified buffer view for the binding, array element and descriptor type
  pub fn bufferview(&mut self, binding: u32, array_elem: u32, ty: vk::DescriptorType, view: vk::BufferView) {
    self.bufferviews(binding, array_elem, ty, &[view]);
  }

  /// Sets the sepecified buffer view for the binding, array element and descriptor type
  pub fn bufferviews(&mut self, binding: u32, array_elem: u32, ty: vk::DescriptorType, views: &[vk::BufferView]) {
    self.writes.push(vk::WriteDescriptorSet {
      sType: vk::STRUCTURE_TYPE_WRITE_DESCRIPTOR_SET,
      pNext: std::ptr::null(),
      dstSet: self.dset,
      dstBinding: binding,
      dstArrayElement: array_elem,
      descriptorType: ty,
      descriptorCount: views.len() as u32,
      pBufferInfo: std::ptr::null(),
      pImageInfo: std::ptr::null(),
      pTexelBufferView: std::ptr::null(),
    });

    debug_assert!(ty & (vk::DESCRIPTOR_TYPE_UNIFORM_TEXEL_BUFFER | vk::DESCRIPTOR_TYPE_STORAGE_TEXEL_BUFFER) != 0);

    self.write_offsets.push(WriteOffset::BufferView(self.buffer_views.len() as isize));
    for view in views.iter() {
      self.buffer_views.push(*view);
    }
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

    vk::UpdateDescriptorSets(self.device, self.writes.len() as u32, self.writes.as_ptr(), 0, std::ptr::null());
  }
}
