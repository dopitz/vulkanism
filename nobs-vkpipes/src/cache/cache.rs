use super::*;
use std::collections::HashMap;

pub struct Cache<T: PipelineId> {
  pipes: HashMap<T, CachedPipeline>,
}

impl<T: PipelineId> Cache<T> {
  pub fn new(info: &T::CreateInfo) -> Self {
    let mut tmp = T::ids().iter().map(|id| (*id, id.create_pipeline(info))).collect::<HashMap<_, _>>();
    let mut pipes = HashMap::with_capacity(tmp.len());
    for id in T::ids().iter() {
      id.setup_dsets(info, &mut tmp, &mut pipes);
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
