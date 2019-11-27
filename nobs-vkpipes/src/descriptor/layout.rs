use vk;

use crate::builder::*;
use crate::pipeline::Binding;
use crate::DescriptorSizes;

/// Descriptor layout with acompanying pool sizes
#[derive(Debug, Clone, Copy)]
pub struct DescriptorLayout {
  pub layout: vk::DescriptorSetLayout,
  pub sizes: DescriptorSizes,
}

impl DescriptorLayout {
  pub fn from_bindings(device: vk::Device, bindings: &[Binding]) -> Self {
    Self {
      layout: create_descriptor_layout(device, bindings),
      sizes: DescriptorSizes::from_bindings(bindings),
    }
  }
}
