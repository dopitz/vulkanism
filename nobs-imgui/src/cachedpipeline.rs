use vk;
use vk::pipes::*;

use std::sync::Arc;

pub struct CachedPipeline {
  pub pipe: Pipeline,
  pub pool: DescriptorPool,
  pub shared: Option<(DescriptorPool, Vec<vk::DescriptorSet>)>,
}

pub type CachedPipelineArc = Arc<CachedPipeline>;

pub trait CacheablePipeline {
  fn create_cache(device: vk::Device, pass: vk::RenderPass, subpass: u32) -> CachedPipeline;
  fn from_cache(pipe: CachedPipelineArc) -> Self;
  fn get_ident() -> &'static str;
  fn get_cache(&self) -> CachedPipelineArc;
}

