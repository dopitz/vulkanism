pub mod builder;

use crate::DescriptorLayout;
use vk;

/// Binding for a uniform variable in a vulkan pipeline
#[derive(Debug, Clone, Copy)]
pub struct Binding {
  pub name: &'static str,
  pub binding: u32,
  pub descset: u32,
  pub desctype: vk::DescriptorType,
  pub stageflags: vk::ShaderStageFlagBits,
}

/// A managed vulkan pipeline object.
///
/// Tracks the lifetime of the vk pipeline with it's acompanying descriptor set layouts and pipeline layouts.
pub struct Pipeline {
  pub device: vk::Device,
  pub handle: vk::Pipeline,
  pub dsets: Vec<DescriptorLayout>,
  pub layout: vk::PipelineLayout,
}

impl Drop for Pipeline {
  /// Cleans up the pipeline, the pipeline layout and all descriptor set layouts
  fn drop(&mut self) {
    for ds in self.dsets.iter() {
      vk::DestroyDescriptorSetLayout(self.device, ds.layout, std::ptr::null());
    }
    vk::DestroyPipelineLayout(self.device, self.layout, std::ptr::null());
    vk::DestroyPipeline(self.device, self.handle, std::ptr::null());
  }
}
