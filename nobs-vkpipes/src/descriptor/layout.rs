use vk;

use crate::DescriptorSizes;

/// Descriptor layout with acompanying pool sizes
#[derive(Debug, Clone, Copy)]
pub struct DescriptorLayout {
  pub layout: vk::DescriptorSetLayout,
  pub sizes: DescriptorSizes,
}
