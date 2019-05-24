use vk;
use vk::builder::Buildable;
use vk::pipes::DescriptorPool;

use crate::cachedpipeline::*;

mod pipe {
  vk::pipes::pipeline! {
    stage = {
      ty = "vert",
      glsl = "src/sprite/sprite.vert",
    }

    stage = {
      ty = "frag",
      glsl = "src/sprite/sprite.frag",
    }

    dset_name[0] = "DsViewport",
    dset_name[1] = "DsText",
  }

  #[derive(Debug)]
  #[repr(C)]
  pub struct Vertex {
    pub pos: vkm::Vec2f,
    pub size: vkm::Vec2f,
    pub tex_bl: vkm::Vec2f,
    pub tex_tr: vkm::Vec2f,
  }

  impl crate::font::FontChar for Vertex {
    fn set_position(&mut self, p: vkm::Vec2f) {
      self.pos = p;
    }
    fn set_size(&mut self, s: vkm::Vec2f) {
      self.size = s;
    }
    fn set_tex(&mut self, t00: vkm::Vec2f, t11: vkm::Vec2f) {
      self.tex_bl = t00;
      self.tex_tr = t11;
    }
  }

  #[repr(C)]
  pub struct Ub {
    pub offset: vkm::Vec2i,
  }

  pub const IDENT: &str = module_path!();
}

#[derive(Clone)]
pub struct Pipeline {
  pipe: CachedPipelineArc,
}

impl CacheablePipeline for Pipeline {
  fn create_cache(device: vk::Device, pass: vk::RenderPass, subpass: u32) -> CachedPipeline {
    let pipe = pipe::new(device, pass, subpass)
      .vertex_input(
        vk::PipelineVertexInputStateCreateInfo::build()
          .push_binding(
            vk::VertexInputBindingDescription::build()
              .binding(0)
              .input_rate(vk::VERTEX_INPUT_RATE_INSTANCE)
              .stride(std::mem::size_of::<pipe::Vertex>() as u32)
              .binding,
          )
          .push_attribute(
            vk::VertexInputAttributeDescription::build()
              .binding(0)
              .location(0)
              .format(vk::FORMAT_R32G32_SFLOAT)
              .attribute,
          )
          .push_attribute(
            vk::VertexInputAttributeDescription::build()
              .binding(0)
              .location(1)
              .format(vk::FORMAT_R32G32_SFLOAT)
              .offset(2 * std::mem::size_of::<f32>() as u32)
              .attribute,
          )
          .push_attribute(
            vk::VertexInputAttributeDescription::build()
              .binding(0)
              .location(2)
              .format(vk::FORMAT_R32G32_SFLOAT)
              .offset(4 * std::mem::size_of::<f32>() as u32)
              .attribute,
          )
          .push_attribute(
            vk::VertexInputAttributeDescription::build()
              .binding(0)
              .location(3)
              .format(vk::FORMAT_R32G32_SFLOAT)
              .offset(6 * std::mem::size_of::<f32>() as u32)
              .attribute,
          ),
      )
      .input_assembly(vk::PipelineInputAssemblyStateCreateInfo::build().topology(vk::PRIMITIVE_TOPOLOGY_TRIANGLE_STRIP))
      .dynamic(
        vk::PipelineDynamicStateCreateInfo::build()
          .push_state(vk::DYNAMIC_STATE_VIEWPORT)
          .push_state(vk::DYNAMIC_STATE_SCISSOR),
      )
      .blend(vk::PipelineColorBlendStateCreateInfo::build().push_attachment(vk::PipelineColorBlendAttachmentState::build()))
      .create()
      .unwrap();

    let pool = DescriptorPool::new(device, DescriptorPool::new_capacity().add(&pipe.dsets[0], 1).add(&pipe.dsets[1], 1));
    let shared = DescriptorPool::new(device, DescriptorPool::new_capacity().add(&pipe.dsets[0], 1));
    let shared_dset = pool.new_dset(&pipe.dsets[0]).unwrap();

    CachedPipeline {
      pipe,
      pool,
      shared: Some((shared, vec![shared_dset])),
    }
  }
  fn from_cache(pipe: CachedPipelineArc) -> Self {
    Self { pipe }
  }
  fn get_ident() -> &'static str {
    pipe::IDENT
  }

  fn get_cache(&self) -> CachedPipelineArc {
    self.pipe.clone()
  }
}

impl Pipeline {
  pub fn new_ds_instance(&mut self) -> Result<vk::DescriptorSet, vk::pipes::Error> {
    self.pipe.pool.new_dset(&self.pipe.pipe.dsets[1])
  }
  pub fn free_ds(&mut self, dset: vk::DescriptorSet) {
    self.pipe.pool.free_dset(dset);
  }
}

pub use pipe::DsText;
pub use pipe::DsViewport;
pub use pipe::Ub;
pub use pipe::Vertex;
