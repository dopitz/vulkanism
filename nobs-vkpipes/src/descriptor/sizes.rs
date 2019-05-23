use vk;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DescriptorSizes {
  counts: [u32; 12],
  pub num_sets: u32,
}

impl std::ops::Index<vk::DescriptorType> for DescriptorSizes {
  type Output = u32;
  fn index(&self, dt: vk::DescriptorType) -> &u32 {
    self.counts.index(Self::get_index(dt))
  }
}
impl std::ops::IndexMut<vk::DescriptorType> for DescriptorSizes {
  fn index_mut(&mut self, dt: vk::DescriptorType) -> &mut u32 {
    self.counts.index_mut(Self::get_index(dt))
  }
}

impl DescriptorSizes {
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

  pub fn new() -> Self {
    unsafe { std::mem::zeroed() }
  }

  pub fn from_pool_sizes(sizes: &[vk::DescriptorPoolSize], num_sets: u32) -> Self {
    let mut ps = Self::new();
    for s in sizes.iter() {
      ps[s.typ] = s.descriptorCount;
    }
    ps.num_sets = num_sets;
    ps
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

  pub fn iter(&self) -> std::slice::Iter<u32> {
    self.counts.iter()
  }

  pub fn iter_mut(&mut self) -> std::slice::IterMut<u32> {
    self.counts.iter_mut()
  }
}
