mod color {
  vk::pipes::pipeline! {
    stage = {
      ty = "vert",
      glsl = "src/style/simple/pipeline/color.vert",
    }

    stage = {
      ty = "frag",
      glsl = "src/style/simple/pipeline/color.frag",
    }

    dset_name[0] = "DsViewport",
    dset_name[1] = "DsStyle",
  }
}
mod select {
  vk::pipes::pipeline! {
    stage = {
      ty = "vert",
      glsl = "src/style/simple/pipeline/select.vert",
    }

    stage = {
      ty = "frag",
      glsl = "src/style/simple/pipeline/select.frag",
    }

    dset_name[0] = "DsViewport",
    dset_name[1] = "DsStyle",
  }
}

#[repr(C)]
#[derive(Debug)]
pub struct UbStyle {
  pub position: vkm::Vec2i,
  pub size: vkm::Vec2i,
  pub bd_thickness: vkm::Vec2i,
  pub id_body: u32,
  pub id_border: u32,
}

use vk;
use vk::builder::Buildable;
use vk::cmd::commands::BindDset;
use vk::cmd::commands::BindPipeline;
use vk::cmd::stream::*;
use vk::cmd::CmdBuffer;

pub struct Pipeline {
  pub color: vk::pipes::Pipeline,
  pub select: vk::pipes::Pipeline,
  pub pool: vk::pipes::DescriptorPool,
}

#[derive(Debug)]
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
  pub fn new(device: vk::Device, pass_draw: vk::RenderPass, subpass_draw: u32, pass_select: vk::RenderPass, subpass_select: u32) -> Self {
    let color = color::new(device, pass_draw, subpass_draw)
      .input_assembly(vk::PipelineInputAssemblyStateCreateInfo::build().topology(vk::PRIMITIVE_TOPOLOGY_TRIANGLE_STRIP))
      .blend(vk::PipelineColorBlendStateCreateInfo::build().push_attachment(
        vk::PipelineColorBlendAttachmentState::build().enable(vk::TRUE).color_and_alpha(
          vk::BLEND_FACTOR_SRC_ALPHA,
          vk::BLEND_FACTOR_ONE_MINUS_SRC_ALPHA,
          vk::BLEND_OP_ADD,
        ),
      ))
      .dynamic(
        vk::PipelineDynamicStateCreateInfo::build()
          .push_state(vk::DYNAMIC_STATE_VIEWPORT)
          .push_state(vk::DYNAMIC_STATE_SCISSOR),
      )
      .create()
      .unwrap();

    let select = select::new(device, pass_select, subpass_select)
      .input_assembly(vk::PipelineInputAssemblyStateCreateInfo::build().topology(vk::PRIMITIVE_TOPOLOGY_TRIANGLE_STRIP))
      .blend(vk::PipelineColorBlendStateCreateInfo::build().push_attachment(vk::PipelineColorBlendAttachmentState::build()))
      .dynamic(
        vk::PipelineDynamicStateCreateInfo::build()
          .push_state(vk::DYNAMIC_STATE_VIEWPORT)
          .push_state(vk::DYNAMIC_STATE_SCISSOR),
      )
      .create()
      .unwrap();

    let pool = vk::pipes::DescriptorPool::new(
      device,
      vk::pipes::DescriptorPool::new_capacity()
        .add(&color.dsets[1], 32)
        .add(&select.dsets[1], 32),
    );

    Self { color, select, pool }
  }

  pub fn new_instance(&mut self, ds_viewport: vk::DescriptorSet, ub_style: vk::Buffer) -> (Bind, Bind) {
    // we only need one descriptor set for ds_style, because it is semantically the same in color and select pipeline
    let ds_style = self.pool.new_dset(&self.color.dsets[1]).unwrap();
    color::DsStyle::write(self.color.device, ds_style)
      .ub(vk::DescriptorBufferInfo::build().buffer(ub_style).into())
      .update();

    (
      self.new_instance_inner(&self.color, ds_viewport, ds_style),
      self.new_instance_inner(&self.select, ds_viewport, ds_style),
    )
  }

  fn new_instance_inner(&self, pipe: &vk::pipes::Pipeline, ds_viewport: vk::DescriptorSet, ds_style: vk::DescriptorSet) -> Bind {
    Bind {
      bind_pipe: BindPipeline::graphics(pipe.handle),
      bind_ds_viewport: BindDset::new(vk::PIPELINE_BIND_POINT_GRAPHICS, pipe.layout, 0, ds_viewport),
      bind_ds_style: BindDset::new(vk::PIPELINE_BIND_POINT_GRAPHICS, pipe.layout, 1, ds_style),
    }
  }
}
