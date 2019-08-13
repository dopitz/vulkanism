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

pub struct Pipeline {
  pub pipe: vk::pipes::Pipeline,

  pub bind_pipe: BindPipeline,
  pub bind_ds_viewport: BindDset,
}

impl StreamPush for Pipeline {
  fn enqueue(&self, cs: CmdBuffer) -> CmdBuffer {
    cs.push(&self.bind_pipe).push(&self.bind_ds_viewport)
  }
}

impl Pipeline {
  pub fn new(device: vk::Device, pass: vk::RenderPass, subpass: u32, ds_viewport: vk::DescriptorSet) -> Self {
    let pipe = pipe::new(device, pass, subpass)
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
      .unwrap();

    let bind_pipe = BindPipeline::graphics(pipe.handle);
    let bind_ds_viewport = BindDset::new(vk::PIPELINE_BIND_POINT_GRAPHICS, pipe.layout, 0, ds_viewport);
    Self {
      pipe,
      bind_pipe,
      bind_ds_viewport,
    }
  }
}
