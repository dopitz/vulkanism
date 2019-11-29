use vk;

use crate::builder::*;
use crate::pipeline::Binding;
use crate::DescriptorSizes;

/// Descriptor layout with acompanying pool sizes
#[derive(Debug, Clone)]
pub struct DescriptorLayout {
  pub layout: vk::DescriptorSetLayout,
  pub sizes: DescriptorSizes,
  pub bindings: Vec<Binding>,
}

impl DescriptorLayout {
  pub fn from_bindings(device: vk::Device, bindings: &[Binding]) -> Self {
    let mut b : Vec<_> = bindings.into();
    b.sort_by_key(|a| a.binding);
    Self {
      layout: create_descriptor_layout(device, bindings),
      sizes: DescriptorSizes::from_bindings(bindings),
      bindings: b,
    }
  }
}
