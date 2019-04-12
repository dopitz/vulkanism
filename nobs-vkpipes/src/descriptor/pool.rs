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

mod x {

  //  #[derive(Clone)]
  //  pub struct PoolSizes {
  //    sizes: Vec<vk::DescriptorPoolSize>,
  //    num_sets: u32,
  //  }
  //
  //  impl std::ops::Index<usize> for PoolSizes {
  //    type Output = vk::DescriptorPoolSize;
  //    fn index(&self, i: usize) -> &vk::DescriptorPoolSize {
  //      self.index(i)
  //    }
  //  }
  //  impl std::ops::IndexMut<usize> for PoolSizes {
  //    fn index_mut(&mut self, i: usize) -> &mut vk::DescriptorPoolSize {
  //      self.index_mut(i)
  //    }
  //  }
  //
  //  impl PartialOrd for PoolSizes {
  //    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
  //      match self
  //        .sizes
  //        .iter()
  //        .zip(other.sizes.iter())
  //        .find(|(s, o)| s.typ != o.typ || s.descriptorCount != o.descriptorCount)
  //      {
  //        None => Some(std::cmp::Ordering::Equal),
  //        Some((s, o)) => {
  //          if s.typ == o.typ {
  //            Some(s.descriptorCount.cmp(&o.descriptorCount))
  //          } else {
  //            Some(std::cmp::Ordering::Equal)
  //          }
  //        }
  //      }
  //    }
  //  }
  //  impl Ord for PoolSizes {
  //    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
  //      self.partial_cmp(other).unwrap()
  //    }
  //  }
  //
  //  impl PartialEq for PoolSizes {
  //    fn eq(&self, other: &Self) -> bool {
  //      for (s, o) in self.sizes.iter().zip(other.sizes.iter()) {
  //        if s.typ != o.typ || s.descriptorCount != o.descriptorCount {
  //          return false;
  //        }
  //      }
  //      return true;
  //    }
  //  }
  //  impl Eq for PoolSizes {}
  //
  //  impl PoolSizes {
  //    pub fn new() -> Self {
  //      PoolSizes {
  //        sizes: Default::default(),
  //        num_sets: 0,
  //      }
  //    }
  //
  //    pub fn len(&self) -> usize {
  //      self.sizes.len()
  //    }
  //
  //    pub fn can_add(&self, other: &PoolSizes) -> bool {
  //      true
  //    }
  //
  //    pub fn add(&mut self, poolsize: vk::DescriptorPoolSize) {
  //      match self.sizes.binary_search_by_key(&poolsize.typ, |s| s.typ) {
  //        Ok(i) => self.sizes[i].descriptorCount += poolsize.descriptorCount,
  //        Err(i) => self.sizes.insert(i, poolsize),
  //      }
  //    }
  //
  //    pub fn add_many(mut self, poolsize: &[vk::DescriptorPoolSize]) {
  //      for s in poolsize.iter() {
  //        self.add(*s);
  //      }
  //    }
  //  }

  #[derive(Clone, PartialEq, Eq, Hash)]
  pub struct PoolSizes {
    counts: [u32; 12],
    pub num_sets: u32,
  }

  impl std::ops::Index<vk::DescriptorType> for PoolSizes {
    type Output = u32;
    fn index(&self, dt: vk::DescriptorType) -> &u32 {
      self.counts.index(Self::get_index(dt))
    }
  }
  impl std::ops::IndexMut<vk::DescriptorType> for PoolSizes {
    fn index_mut(&mut self, dt: vk::DescriptorType) -> &mut u32 {
      self.counts.index_mut(Self::get_index(dt))
    }
  }

  impl PoolSizes {
    const DESCTYPE_X: vk::DescriptorType = vk::DESCRIPTOR_TYPE_INPUT_ATTACHMENT;

    fn get_index(dt: vk::DescriptorType) -> usize {
      if dt <= Self::DESCTYPE_X {
        dt as usize
      } else {
        match dt {
          vk::DESCRIPTOR_TYPE_INLINE_UNIFORM_BLOCK_EXT => (Self::DESCTYPE_X + 1) as usize,
          vk::DESCRIPTOR_TYPE_ACCELERATION_STRUCTURE_NV => (Self::DESCTYPE_X + 2) as usize,
          _ => panic!("invalid descriptor type index"),
        }
      }
    }

    fn get_desctype(i: usize) -> vk::DescriptorType {
      let i = i as vk::DescriptorType;
      if i <= Self::DESCTYPE_X {
        i
      } else if i == Self::DESCTYPE_X + 1 {
        vk::DESCRIPTOR_TYPE_INLINE_UNIFORM_BLOCK_EXT
      } else if i == Self::DESCTYPE_X + 2 {
        vk::DESCRIPTOR_TYPE_ACCELERATION_STRUCTURE_NV
      } else {
        panic!("invalid descriptor type index");
      }
    }

    pub fn to_pool_sizes(&self) -> Vec<vk::DescriptorPoolSize> {
      self
        .counts
        .iter()
        .enumerate()
        .filter_map(|(i, c)| {
          if c != &0 {
            Some(vk::DescriptorPoolSize {
              typ: Self::get_desctype(i),
              descriptorCount: *c,
            })
          } else {
            None
          }
        })
        .collect()
    }

    pub fn new() -> Self {
      unsafe { std::mem::zeroed() }
    }

    pub fn from_pool_sizes(sizes: &[vk::DescriptorPoolSize]) -> Self {
      let mut ps = Self::new();
      for s in sizes.iter() {
        ps[s.typ] = s.descriptorCount;
      }
      ps
    }
  }

  pub struct Pool {
    pub device: vk::Device,
    capacity: PoolSizes,
    capacity_vec: Vec<vk::DescriptorPoolSize>,
    pools: std::collections::HashMap<vk::DescriptorPool, PoolSizes>,

    dsets: std::collections::HashMap<vk::DescriptorSet, (vk::DescriptorPool, usize)>,

    dset_types: Vec<(PoolSizes, usize)>,
  }

  impl Pool {
    fn create_pool(&mut self) -> Result<vk::DescriptorPool, vk::Error> {
      // create new pool
      let create_info = vk::DescriptorPoolCreateInfo {
        sType: vk::STRUCTURE_TYPE_DESCRIPTOR_POOL_CREATE_INFO,
        pNext: std::ptr::null(),
        flags: 0,
        poolSizeCount: self.capacity_vec.len() as u32,
        pPoolSizes: self.capacity_vec.as_ptr(),
        maxSets: self.capacity.num_sets,
      };
      let mut handle = vk::NULL_HANDLE;
      vk_check!(vk::CreateDescriptorPool(self.device, &create_info, std::ptr::null(), &mut handle))?;

      self.pools.insert(handle, PoolSizes::new());
      Ok(handle)
    }

    //pub fn new(device: vk::Device, capacity: PoolSizes) -> Self {
    //  Pool {
    //    device,
    //    capacity,
    //    pools: Default::default(),
    //    dsets: Default::default(),
    //  }
    //}

    pub fn new_dest(&mut self, layout: vk::DescriptorSetLayout, sizes: &PoolSizes) -> Result<vk::DescriptorSet, vk::Error> {
      if !self.capacity.counts.iter().zip(sizes.counts.iter()).fold(true, |acc, (c, s)| acc && s <= c) {
        panic!("AOEUAOEU");
      }

      let pool = match self.pools.iter().find(|(p, c)| c.can_add(&sizes)) {
        Some((p, c)) => *p,
        None => self.create_pool()?,
      };

      let create_info = vk::DescriptorSetAllocateInfo {
        sType: vk::STRUCTURE_TYPE_DESCRIPTOR_SET_ALLOCATE_INFO,
        pNext: std::ptr::null(),
        descriptorPool: pool,
        descriptorSetCount: 1,
        pSetLayouts: &layout,
      };

      let mut handle = vk::NULL_HANDLE;
      vk_check!(vk::AllocateDescriptorSets(self.device, &create_info, &mut handle))?;

      let dset_sizes = match self.dset_types.binary_search_by(|(s, id)| sizes.cmp(s)) {
        Ok(i) => &self.dset_types[i],
        Err(i) => {
          self.dset_types.insert(i, (dset_sizes, self.dset_types.len()));
          &self.dset_types[i]
        }
      };

      let pool_sizes = self.pools.get_mut(&pool).unwrap();

      Ok(handle)
    }

    //pub fn free_dset(&mut self, dset: vk::DescriptorSet) {
    //  if let Some((p, dt)) = self.dsets.remove(&dset) {
    //    vk_uncheck!(vk::FreeDescriptorSets(self.device, p, 1, &dset));

    //    let poll_sizes = self.pools.get(&p).unwrap();
    //    let dset_sizes = self.dset_types.iter().find(|(s, d)| *d == dt).unwrap();
    //  }
    //}
  }
}
