use std::collections::HashMap;

use vk;
use vk::pipes::Cache;
use vk::pipes::CachedPipeline;
use vk::pipes::PipelineId;

use crate::sprites;

#[derive(Hash, Clone, Copy, PartialEq, Eq)]
pub enum PipeId {
  Sprites,
}

pub struct PipeCreateInfo {
  pub device: vk::Device,
  pub pass: vk::RenderPass,
  pub subpass: u32,
  pub ub_viewport: vk::Buffer,
}

impl PipelineId for PipeId {
  type CreateInfo = PipeCreateInfo;
  fn create_pipeline(&self, info: &Self::CreateInfo) -> vk::pipes::Pipeline {
    match self {
      PipeId::Sprites => sprites::Pipeline::create_pipeline(info.device, info.pass, info.subpass),
    }
  }

  fn setup_dsets(
    &self,
    info: &Self::CreateInfo,
    pipes: &mut HashMap<Self, vk::pipes::Pipeline>,
    cache: &mut HashMap<Self, CachedPipeline>,
  ) {
    match self {
      PipeId::Sprites => {
        cache.insert(
          PipeId::Sprites,
          sprites::Pipeline::setup_dsets(pipes.remove(&PipeId::Sprites).unwrap(), info.ub_viewport),
        );
      }
    }
  }

  fn ids() -> Vec<Self> {
    vec![PipeId::Sprites]
  }
}

pub type PipeCache = Cache<PipeId>;