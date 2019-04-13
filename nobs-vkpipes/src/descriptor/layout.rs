use vk;

use crate::descriptor::PoolSizes;

/// Descriptor layout with acompanying pool sizes
#[derive(Debug, Clone, Copy)]
pub struct Layout {
  pub layout: vk::DescriptorSetLayout,
  pub sizes: PoolSizes,
}
