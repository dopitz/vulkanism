use std::collections::HashMap;

use vk;
use vk::pipes::Cache;
use vk::pipes::CachedPipeline;
use vk::pipes::PipelineId;

use crate::sprites;
use crate::select::rects;

#[derive(Hash, Clone, Copy, PartialEq, Eq)]
pub enum PipeId {
  Sprites,
  SelectRects,
}

pub struct PipeCreateInfo {
  pub device: vk::Device,
  pub pass: vk::RenderPass,
  pub subpass: u32,
  pub ub_viewport: vk::Buffer,

  pub select_pass: vk::RenderPass,
  pub select_subpass: u32,
}

impl PipelineId for PipeId {
  type CreateInfo = PipeCreateInfo;
  fn create_pipeline(&self, info: &Self::CreateInfo) -> vk::pipes::Pipeline {
    match self {
      PipeId::Sprites => sprites::Pipeline::create_pipeline(info.device, info.pass, info.subpass),
      PipeId::SelectRects => rects::Pipeline::create_pipeline(info.device, info.select_pass, info.select_subpass),
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
      PipeId::SelectRects => {
        cache.insert(
          PipeId::SelectRects,
          rects::Pipeline::setup_dsets(pipes.remove(&PipeId::SelectRects).unwrap(), info.ub_viewport),
        );
      }
    }
  }

  fn ids() -> Vec<Self> {
    vec![PipeId::Sprites, PipeId::SelectRects]
  }
}

pub type PipeCache = Cache<PipeId>;
