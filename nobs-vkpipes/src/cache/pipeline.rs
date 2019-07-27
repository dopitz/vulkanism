use crate::DescriptorPool;
use crate::Pipeline;
use std::collections::HashMap;
use vk;

/// Trait to use pipelines from a [Cache](struct.Cache.html)
///
/// This trait is supposed to be implemented for enum types.
/// For every enum variant the [create_pipeline](trait.PipelineId.html#method.create_pipeline) 
/// and [setup_dsets](trait.PipelineId.html#method.setup_dsets) methods need to be implemented.
pub trait PipelineId: std::hash::Hash + PartialEq + Eq + Clone + Copy {
  /// Additional info that can be used in [create_pipeline](trait.PipelineId.html#method.create_pipeline) and [setup_dsets](trait.PipelineId.html#method.setup_dsets)
  type CreateInfo;

  /// Creates a pipeline
  ///
  /// This function is invoked for every variant returned from [ids](trait.PipelineId.html#method.ids) on creation of the pipeline cache.
  ///
  /// # Returns
  /// The created pipeline for the variant of `self`
  fn create_pipeline(&self, info: &Self::CreateInfo) -> Pipeline;
  /// Sets up descritpor pools and descriptor sets for every variant in [ids](trait.PipelineId.html#method.ids).
  /// 
  /// The variant of self should be removed from `pipes` and inserted into `cache`.
  /// We use this two step setup with pipeline creation and descriptor pool setup, so that different pipelines can share the same descriptor pool.
  fn setup_dsets(&self, info: &Self::CreateInfo, pipes: &mut HashMap<Self, Pipeline>, cache: &mut HashMap<Self, CachedPipeline>);

  /// Returns all variants of the PipelineId
  fn ids() -> Vec<Self>;
}

/// Cached pipeline and destriptor pools
///
/// Comrises the pipeline itself and optionally a descriptor pool and a shared descriptor pool
pub struct CachedPipeline {
  /// The actual pipelne
  pub pipe: Pipeline,
  /// Descriptor pool containing descriptor sets only valid for this pipeline.
  pub dsets: Option<DescriptorPool>,
  /// Destriptor pool containing descritpor sets valid for this and other pipelines 
  pub dsets_shared: Option<(DescriptorPool, Vec<vk::DescriptorSet>)>,
}
