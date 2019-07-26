mod pipe {
  vk::pipes::pipeline! {
    stage = {
      ty = "vert",
      glsl = "src/select/rects/rects.vert",
    }

    stage = {
      ty = "frag",
      glsl = "src/select/rects/rects.frag",
    }

    dset_name[0] = "DsViewport",
  }
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct Vertex {
  pub pos: vkm::Vec2f,
  pub size: vkm::Vec2f,
  pub id: u32,
}

use vk;
use vk::builder::Buildable;
use vk::cmd::commands::BindDset;
use vk::cmd::commands::BindPipeline;
use vk::cmd::stream::*;
use vk::cmd::CmdBuffer;
use vk::pipes::CachedPipeline;
use vk::pipes::DescriptorPool;

pub struct Pipeline {
  pub bind_pipe: BindPipeline,
  pub bind_ds_viewport: BindDset,
}

impl StreamPush for Pipeline {
  fn enqueue(&self, cs: CmdBuffer) -> CmdBuffer {
    cs.push(&self.bind_pipe).push(&self.bind_ds_viewport)
  }
}

impl Pipeline {
  pub fn create_pipeline(device: vk::Device, pass: vk::RenderPass, subpass: u32) -> vk::pipes::Pipeline {
    pipe::new(device, pass, subpass)
      .vertex_input(
        vk::PipelineVertexInputStateCreateInfo::build()
          .push_binding(
            vk::VertexInputBindingDescription::build()
              .binding(0)
              .input_rate(vk::VERTEX_INPUT_RATE_INSTANCE)
              .stride(std::mem::size_of::<Vertex>() as u32),
          )
          .push_attribute(
            vk::VertexInputAttributeDescription::build()
              .binding(0)
              .location(0)
              .format(vk::FORMAT_R32G32_SFLOAT),
          )
          .push_attribute(
            vk::VertexInputAttributeDescription::build()
              .binding(0)
              .location(1)
              .format(vk::FORMAT_R32G32_SFLOAT)
              .offset(2 * std::mem::size_of::<f32>() as u32),
          )
          .push_attribute(
            vk::VertexInputAttributeDescription::build()
              .binding(0)
              .location(2)
              .format(vk::FORMAT_R32_UINT)
              .offset(4 * std::mem::size_of::<f32>() as u32),
          ),
      )
      .input_assembly(vk::PipelineInputAssemblyStateCreateInfo::build().topology(vk::PRIMITIVE_TOPOLOGY_TRIANGLE_STRIP))
      .blend(vk::PipelineColorBlendStateCreateInfo::build().push_attachment(vk::PipelineColorBlendAttachmentState::build()))
      .dynamic(
        vk::PipelineDynamicStateCreateInfo::build()
          .push_state(vk::DYNAMIC_STATE_VIEWPORT)
          .push_state(vk::DYNAMIC_STATE_SCISSOR),
      )
      .create()
      .unwrap()
  }

  pub fn setup_dsets(pipe: vk::pipes::Pipeline, ub_viewport: vk::Buffer) -> CachedPipeline {
    let dsets = Some(DescriptorPool::new(pipe.device, DescriptorPool::new_capacity().add(&pipe.dsets[0], 1)));
    let shared = DescriptorPool::new(pipe.device, DescriptorPool::new_capacity().add(&pipe.dsets[0], 1));
    let ds_viewport = shared.new_dset(&pipe.dsets[0]).unwrap();

    pipe::DsViewport::write(pipe.device, ds_viewport)
      .ub_viewport(vk::DescriptorBufferInfo::build().buffer(ub_viewport).into())
      .update();

    CachedPipeline {
      pipe,
      dsets,
      dsets_shared: Some((shared, vec![ds_viewport])),
    }
  }

  pub fn new(cache: &CachedPipeline) -> Self {
    Self {
      bind_pipe: BindPipeline::graphics(cache.pipe.handle),
      bind_ds_viewport: BindDset::new(
        vk::PIPELINE_BIND_POINT_GRAPHICS,
        cache.pipe.layout,
        0,
        match cache.dsets_shared {
          Some((_, ref ds)) => ds[0],
          None => panic!("should never happen"),
        },
      ),
    }
  }
}
