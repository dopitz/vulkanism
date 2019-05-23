use crate::DescriptorLayout;
use crate::DescriptorSizes;
use crate::Error;
use vk;

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

/// Builder for [DescriptorSizes](sizes/struct.DescriptorSizes.html).
///
/// Lets one conveniently aggregate the needed number of descriptors by adding [Layouts](layout/struct.DescriptorLayout.html)
pub struct PoolCapacity {
  capacity: DescriptorSizes,
}

impl PoolCapacity {
  /// Adds the [DescriptorSizes](sizes/struct.DescriptorSizes.html) of `layout` to the capacity.
  ///
  /// # Arguments
  /// * `layout` - Specifies the number of descriptors for a SINGLE desciptor set
  /// * `num_sets` - Number of descriptor sets of this type, that may be allocated in a single pool
  ///                Each descriptor count is multiplied by `num_sets` prior to adding to the capacity.
  pub fn add(mut self, layout: &DescriptorLayout, num_sets: u32) -> Self {
    for (c, s) in self.capacity.iter_mut().zip(layout.sizes.iter()) {
      *c += num_sets * s
    }
    self.capacity.num_sets += num_sets;
    self
  }
}

struct PoolImpl {
  device: vk::Device,
  capacity: DescriptorSizes,
  capacity_vec: Vec<vk::DescriptorPoolSize>,
  pools: HashMap<vk::DescriptorPool, DescriptorSizes>,
  dsets: HashMap<vk::DescriptorSet, (vk::DescriptorPool, usize)>,
  dset_types: HashMap<DescriptorSizes, usize>,
}

impl Drop for PoolImpl {
  fn drop(&mut self) {
    for (p, _) in self.pools.iter() {
      vk::DestroyDescriptorPool(self.device, *p, std::ptr::null());
    }
  }
}

#[derive(Clone)]
pub struct DescriptorPool {
  pool: Arc<Mutex<PoolImpl>>,
}

impl DescriptorPool {
  pub fn new_capacity() -> PoolCapacity {
    PoolCapacity {
      capacity: DescriptorSizes::new(),
    }
  }

  pub fn new(device: vk::Device, capacity: PoolCapacity) -> Self {
    let capacity = capacity.capacity;
    let capacity_vec = capacity.to_pool_sizes();
    Self {
      pool: Arc::new(Mutex::new(PoolImpl {
        device,
        capacity,
        capacity_vec,
        pools: Default::default(),
        dsets: Default::default(),
        dset_types: Default::default(),
      })),
    }
  }

  pub fn new_dset(&self, layout: &DescriptorLayout) -> Result<vk::DescriptorSet, crate::Error> {
    let mut pi = self.pool.lock().unwrap();

    // make sure the pools can allocate such a descriptor
    if !pi.capacity.iter().zip(layout.sizes.iter()).fold(true, |acc, (c, s)| acc && s <= c) {
      Err(crate::Error::InvalidDescriptorCount)?;
    }

    // find the first pool with enough space to hold the descriptor
    let pool = match pi.pools.iter().find(|(_, pool_sizes)| {
      let sum = pool_sizes.iter().zip(layout.sizes.iter()).map(|(p, s)| p + s);
      pi.capacity.iter().zip(sum).fold(true, |acc, (cap, sum)| acc && sum <= *cap)
    }) {
      Some((p, _)) => *p,
      None => {
        // create new pool
        let create_info = vk::DescriptorPoolCreateInfo {
          sType: vk::STRUCTURE_TYPE_DESCRIPTOR_POOL_CREATE_INFO,
          pNext: std::ptr::null(),
          flags: vk::DESCRIPTOR_POOL_CREATE_FREE_DESCRIPTOR_SET_BIT,
          poolSizeCount: pi.capacity_vec.len() as u32,
          pPoolSizes: pi.capacity_vec.as_ptr(),
          maxSets: pi.capacity.num_sets,
        };
        let mut handle = vk::NULL_HANDLE;
        vk_check!(vk::CreateDescriptorPool(pi.device, &create_info, std::ptr::null(), &mut handle))
          .map_err(|e| Error::DescriptorPoolCreateFail(e))?;

        pi.pools.insert(handle, DescriptorSizes::new());
        handle
      }
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
    vk_check!(vk::AllocateDescriptorSets(pi.device, &create_info, &mut handle)).map_err(|e| Error::DescriptorSetCreateFail(e))?;

    // update the descriptor counts in the pool
    let pool_sizes = pi.pools.get_mut(&pool).unwrap();
    for (c, s) in pool_sizes.iter_mut().zip(layout.sizes.iter()) {
      *c += s;
    }

    // register descriptor set
    let id = pi.dset_types.len();
    let id = *pi.dset_types.entry(layout.sizes).or_insert(id);
    pi.dsets.insert(handle, (pool, id));

    Ok(handle)
  }

  pub fn free_dset(&self, dset: vk::DescriptorSet) {
    let mut pi = self.pool.lock().unwrap();

    if let Some((p, id)) = pi.dsets.remove(&dset) {
      vk_uncheck!(vk::FreeDescriptorSets(pi.device, p, 1, &dset));

      let dset_sizes = pi.dset_types.iter().find(|(_, i)| **i == id).map(|(sizes, _)| sizes).cloned().unwrap();
      let pool_sizes = pi.pools.get_mut(&p).unwrap();
      for (c, s) in pool_sizes.iter_mut().zip(dset_sizes.iter()) {
        *c -= s;
      }
    }
  }
}
