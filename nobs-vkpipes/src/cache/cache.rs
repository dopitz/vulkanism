use super::*;
use std::collections::HashMap;

/// Collection of cached vulkan pipelines.
///
/// With a user implemented [PipelineId](trait.PipelineId.html)
/// this cache will create pipelines once and makes them easily accassible through [CachedPipeline](struct.CachedPipeline.html)
pub struct Cache<T: PipelineId> {
  pipes: HashMap<T, CachedPipeline>,
}

impl<T: PipelineId> Cache<T> {
  /// Creates the pipeline cache
  ///
  /// This will create pipelines and setup descriptor pools as defined in the user specified [PipelineId](trait.PipelineId.html)
  pub fn new(info: &T::CreateInfo) -> Self {
    let mut tmp = T::ids().iter().map(|id| (*id, id.create_pipeline(info))).collect::<HashMap<_, _>>();
    let mut pipes = HashMap::with_capacity(tmp.len());

    loop {
      if let Some(id) = tmp.keys().next().cloned() {
        for (id, pipe) in id.setup_dsets(info, &mut tmp).into_iter() {
          pipes.insert(id, pipe);
        }
      } else {
        break;
      }
    }

    Self { pipes }
  }
}

impl<T: PipelineId> std::ops::Index<T> for Cache<T> {
  type Output = CachedPipeline;
  fn index(&self, id: T) -> &Self::Output {
    &self.pipes[&id]
  }
}
