use crate::DescriptorPool;
use crate::Pipeline;
use std::collections::HashMap;
use vk;

pub trait PipelineId: std::hash::Hash + PartialEq + Eq + Clone + Copy {
  type CreateInfo;
  fn create_pipeline(&self, info: &Self::CreateInfo) -> Pipeline;
  fn setup_dsets(&self, info: &Self::CreateInfo, pipes: &mut HashMap<Self, Pipeline>, cache: &mut HashMap<Self, CachedPipeline>);

  fn ids() -> Vec<Self>;
}

pub struct CachedPipeline {
  pub pipe: Pipeline,
  pub dsets: DescriptorPool,
  pub dsets_shared: Option<(DescriptorPool, Vec<vk::DescriptorSet>)>,
}
