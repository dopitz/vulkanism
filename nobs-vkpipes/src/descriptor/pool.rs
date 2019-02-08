use vk;

pub type PoolSizes = Vec<vk::DescriptorPoolSize>;

/// Simple descriptor pool, that tracks number of available descriptors for each descriptor type
pub struct Pool {
  pub device: vk::Device,
  pub handle: vk::DescriptorPool,
  pub count: PoolSizes,
  pub count_dsets: u32,
  pub capacity: PoolSizes,
  pub capacity_dsets: u32,
}

impl Drop for Pool {
  fn drop(&mut self) {
    vk::DestroyDescriptorPool(self.device, self.handle, std::ptr::null());
  }
}

impl Pool {
  /// Create a new descriptor pool with the specified pool sizes
  ///
  /// ## Arguments
  /// * `device` - device handle
  /// * `sizes` - maximum number of slots for each destriptor type in the pool.
  /// * `num_sets` - maximum number of descriptor in the pool.
  pub fn with_capacity(device: vk::Device, sizes: &[vk::DescriptorPoolSize], num_sets: u32) -> Result<Pool, vk::Error> {
    // create new pool
    let create_info = vk::DescriptorPoolCreateInfo {
      sType: vk::STRUCTURE_TYPE_DESCRIPTOR_POOL_CREATE_INFO,
      pNext: std::ptr::null(),
      flags: 0,
      poolSizeCount: sizes.len() as u32,
      pPoolSizes: sizes.as_ptr(),
      maxSets: std::cmp::max(1u32, num_sets),
    };
    let mut handle = vk::NULL_HANDLE;
    vk_check!(vk::CreateDescriptorPool(device, &create_info, std::ptr::null(), &mut handle))?;

    Ok(Pool {
      device,
      handle,
      count: Default::default(),
      count_dsets: 0,
      capacity: sizes.to_vec(),
      capacity_dsets: num_sets,
    })
  }

  /// Create a new descriptor set from the pool
  ///
  /// ## Arguments
  /// * `layout` - layout of the destriptor set
  /// * `sizes` - number of slots consumed by this destriptor set
  pub fn new_dset(&mut self, layout: vk::DescriptorSetLayout, sizes: &PoolSizes) -> Result<vk::DescriptorSet, vk::Error> {
    let create_info = vk::DescriptorSetAllocateInfo {
      sType: vk::STRUCTURE_TYPE_DESCRIPTOR_SET_ALLOCATE_INFO,
      pNext: std::ptr::null(),
      descriptorPool: self.handle,
      descriptorSetCount: 1,
      pSetLayouts: &layout,
    };

    let mut handle = vk::NULL_HANDLE;
    vk_check!(vk::AllocateDescriptorSets(self.device, &create_info, &mut handle))?;

    // keep sizes up to date
    for s in sizes.iter() {
      match self.count.binary_search_by_key(&s.typ, |c| c.typ) {
        Ok(i) => self.count[i].descriptorCount += s.descriptorCount,
        Err(i) => self.count.insert(i, *s),
      }
    }
    self.count_dsets += 1;

    Ok(handle)
  }
}
