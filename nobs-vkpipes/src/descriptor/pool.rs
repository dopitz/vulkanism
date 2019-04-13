use crate::descriptor::Layout;
use crate::descriptor::PoolSizes;
use crate::Error;
use vk;

pub struct PoolCapacity {
  capacity: PoolSizes,
}

impl PoolCapacity {
  pub fn add(mut self, layout: &Layout, num_sets: u32) -> Self {
    for (c, s) in self.capacity.iter_mut().zip(layout.sizes.iter()) {
      *c += num_sets * s
    }
    self.capacity.num_sets += num_sets;
    self
  }
}

pub struct Pool {
  pub device: vk::Device,
  capacity: PoolSizes,
  capacity_vec: Vec<vk::DescriptorPoolSize>,
  pools: std::collections::HashMap<vk::DescriptorPool, PoolSizes>,
  dsets: std::collections::HashMap<vk::DescriptorSet, (vk::DescriptorPool, usize)>,
  dset_types: std::collections::HashMap<PoolSizes, usize>,
}

impl Drop for Pool {
  fn drop(&mut self) {
    for (p, _) in self.pools.iter() {
      vk::DestroyDescriptorPool(self.device, *p, std::ptr::null());
    }
  }
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

  pub fn new_capacity() -> PoolCapacity {
    PoolCapacity {
      capacity: PoolSizes::new(),
    }
  }

  pub fn new(device: vk::Device, capacity: PoolCapacity) -> Self {
    let capacity = capacity.capacity;
    let capacity_vec = capacity.to_pool_sizes();
    Pool {
      device,
      capacity,
      capacity_vec,
      pools: Default::default(),
      dsets: Default::default(),
      dset_types: Default::default(),
    }
  }

  pub fn new_dset(&mut self, layout: &Layout) -> Result<vk::DescriptorSet, crate::Error> {
    // make sure the pools can allocate such a descriptor
    if !self
      .capacity
      .iter()
      .zip(layout.sizes.iter())
      .fold(true, |acc, (c, s)| acc && s <= c)
    {
      Err(crate::Error::InvalidDescriptorCount)?;
    }

    // find the first pool with enough space to hold the descriptor
    let pool = match self.pools.iter().find(|(_, pool_sizes)| {
      let sum = pool_sizes.iter().zip(layout.sizes.iter()).map(|(p, s)| p + s);
      self.capacity.iter().zip(sum).fold(true, |acc, (cap, sum)| acc && sum <= *cap)
    }) {
      Some((p, _)) => *p,
      None => self.create_pool().map_err(|e| Error::DescriptorPoolCreateFail(e))?,
    };

    // create the descriptor set
    let create_info = vk::DescriptorSetAllocateInfo {
      sType: vk::STRUCTURE_TYPE_DESCRIPTOR_SET_ALLOCATE_INFO,
      pNext: std::ptr::null(),
      descriptorPool: pool,
      descriptorSetCount: 1,
      pSetLayouts: &layout.layout,
    };

    let mut handle = vk::NULL_HANDLE;
    vk_check!(vk::AllocateDescriptorSets(self.device, &create_info, &mut handle)).map_err(|e| Error::DescriptorSetCreateFail(e))?;

    // update the descriptor counts in the pool
    let pool_sizes = self.pools.get_mut(&pool).unwrap();
    for (c, s) in pool_sizes.iter_mut().zip(layout.sizes.iter()) {
      *c += s;
    }

    // register descriptor set
    let id = self.dset_types.len();
    let id = *self.dset_types.entry(layout.sizes).or_insert(id);
    self.dsets.insert(handle, (pool, id));

    Ok(handle)
  }

  pub fn free_dset(&mut self, dset: vk::DescriptorSet) {
    if let Some((p, id)) = self.dsets.remove(&dset) {
      vk_uncheck!(vk::FreeDescriptorSets(self.device, p, 1, &dset));

      let dset_sizes = self.dset_types.iter().find(|(s, i)| **i == id).map(|(sizes, id)| sizes).unwrap();
      let pool_sizes = self.pools.get_mut(&p).unwrap();
      for (c, s) in pool_sizes.iter_mut().zip(dset_sizes.iter()) {
        *c -= s;
      }
    }
  }
}
