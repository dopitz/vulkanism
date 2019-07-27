use std::collections::HashMap;

use vk;
use vk::pipes::Cache;
use vk::pipes::CachedPipeline;
use vk::pipes::PipelineId;

use crate::sprites;
use crate::select::rects;

/// Pipelino identifiers for gui pipelines
#[derive(Hash, Clone, Copy, PartialEq, Eq)]
pub enum PipeId {
  /// For rendering textured sprites or text
  Sprites,
  /// For rendring object ids into u32 framebuffer 
  SelectRects,
}

/// Additional info for initializing cached pipelines
pub struct PipeCreateInfo {
  /// Vulkan device handle for which pipelines, descriptor pools and descriptors are created
  pub device: vk::Device,

  /// Vulkan renderpass handle of the gui draw pass
  pub pass: vk::RenderPass,
  /// Subpass id of the gui draw pass
  pub subpass: u32,
  /// shared uniform buffer containing the viewport dimensions
  pub ub_viewport: vk::Buffer,

  /// Vulkan renderpass handle of the object selection pass
  pub select_pass: vk::RenderPass,
  /// Subpass id of the object selection pass
  pub select_subpass: u32,
}

impl PipelineId for PipeId {
  type CreateInfo = PipeCreateInfo;

  /// Creates pipelines for [PipeId](enum.PipeId.html)
  fn create_pipeline(&self, info: &Self::CreateInfo) -> vk::pipes::Pipeline {
    match self {
      PipeId::Sprites => sprites::Pipeline::create_pipeline(info.device, info.pass, info.subpass),
      PipeId::SelectRects => rects::Pipeline::create_pipeline(info.device, info.select_pass, info.select_subpass),
    }
  }

  /// Sets up descriptor pools and shared descriptor sets for cached pipelines.
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
