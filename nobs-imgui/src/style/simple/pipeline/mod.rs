mod border;
mod borderless;
mod select;
mod selectborderless;

#[repr(C)]
#[derive(Debug)]
pub struct Vertex {
  pub pos: vkm::Vec2f,
  pub size: vkm::Vec2f,
}

#[repr(C)]
#[derive(Debug)]
pub struct UbStyle {
  pub position: vkm::Vec2i,
  pub size: vkm::Vec2i,
  pub bd_thickness: vkm::Vec2i,
}

use crate::pipelines::PipePool;
use vk;
use vk::builder::Buildable;
use vk::cmd::commands::BindDset;
use vk::cmd::commands::BindPipeline;
use vk::cmd::stream::*;
use vk::cmd::CmdBuffer;

pub struct Pipeline {
  pub border: vk::pipes::Pipeline,
  pub borderless: vk::pipes::Pipeline,
  pub select: vk::pipes::Pipeline,
  pub pool: vk::pipes::DescriptorPool,
}

pub struct Bind {
  pub bind_pipe: BindPipeline,
  pub bind_ds_viewport: BindDset,
  pub bind_ds_style: BindDset,
}

impl StreamPush for Bind {
  fn enqueue(&self, cs: CmdBuffer) -> CmdBuffer {
    cs.push(&self.bind_pipe).push(&self.bind_ds_viewport).push(&self.bind_ds_style)
  }
}

impl Pipeline {
  pub fn new(device: vk::Device, pass: vk::RenderPass, subpass: u32) -> Self {
    let config_pipeline = |mut builder: vk::pipes::pipeline::builder::graphics::Graphics| {
      builder
        .input_assembly(vk::PipelineInputAssemblyStateCreateInfo::build().topology(vk::PRIMITIVE_TOPOLOGY_TRIANGLE_STRIP))
        .dynamic(
          vk::PipelineDynamicStateCreateInfo::build()
            .push_state(vk::DYNAMIC_STATE_VIEWPORT)
            .push_state(vk::DYNAMIC_STATE_SCISSOR),
        )
        .blend(vk::PipelineColorBlendStateCreateInfo::build().push_attachment(
          vk::PipelineColorBlendAttachmentState::build().enable(vk::TRUE).color_and_alpha(
            vk::BLEND_FACTOR_SRC_ALPHA,
            vk::BLEND_FACTOR_ONE_MINUS_SRC_ALPHA,
            vk::BLEND_OP_ADD,
          ),
        ))
        .create()
        .unwrap()
    };

    let border = config_pipeline(border::new(device, pass, subpass));
    let borderless = config_pipeline(border::new(device, pass, subpass));
    let select = config_pipeline(border::new(device, pass, subpass));

    let pool = vk::pipes::DescriptorPool::new(
      device,
      vk::pipes::DescriptorPool::new_capacity()
        .add(&border.dsets[1], 32)
        .add(&select.dsets[1], 32),
    );

    Self {
      border,
      borderless,
      select,
      pool,
    }
  }

  pub fn new_border(&mut self, ds_viewport: vk::DescriptorSet) -> Bind {
    Self::new_instance(&self.border, &mut self.pool, ds_viewport)
  }

  fn new_instance(pipe: &vk::pipes::Pipeline, pool: &mut vk::pipes::DescriptorPool, ds_viewport: vk::DescriptorSet) -> Bind {
    Bind {
      bind_pipe: BindPipeline::graphics(pipe.handle),
      bind_ds_viewport: BindDset::new(vk::PIPELINE_BIND_POINT_GRAPHICS, pipe.layout, 0, ds_viewport),
      bind_ds_style: BindDset::new(
        vk::PIPELINE_BIND_POINT_GRAPHICS,
        pipe.layout,
        1,
        pool.new_dset(&pipe.dsets[1]).unwrap(),
      ),
    }
  }
}

impl Bind {
  pub fn update_dsets(&self, device: vk::Device, ub_style: vk::Buffer) {
    border::DsStyle::write(device, self.bind_ds_style.dset)
      .ub(vk::DescriptorBufferInfo::build().buffer(ub_style).into())
      .update();
  }
}
