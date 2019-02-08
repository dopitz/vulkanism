pub mod pool;
pub mod writes;

use vk;

/// Descriptor layout with acompanying pool sizes
#[derive(Debug, Clone)]
pub struct DsetLayout {
  pub layout: vk::DescriptorSetLayout,
  pub sizes: Vec<vk::DescriptorPoolSize>,
}
