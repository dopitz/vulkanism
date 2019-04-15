use std::sync::Arc;

use vk;
use vk::builder::Buildable;
use vk::cmd;
use vk::pipes::descriptor;

use crate::font::FontID;
use crate::ImGui;

mod pipe {
  vk::pipes::pipeline! {
    stage = {
      ty = "vert",
      glsl = "src/text/text.vert",
    }

    stage = {
      ty = "frag",
      glsl = "src/text/text.frag",
    }

    dset_name[0] = "OnResize",
    dset_name[1] = "Once",
  }

  #[repr(C)]
  pub struct Vertex {
    pub pos: cgm::Vector2<u32>,
    pub size: cgm::Vector2<u32>,
  }
}

pub struct Pipeline {
  pipe: vk::pipes::Pipeline,
  pool: descriptor::Pool,
}

impl Pipeline {
  pub fn new(device: vk::Device, pass: vk::RenderPass, subpass: u32) -> Self {
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
              .format(vk::FORMAT_R32G32_UINT)
              .attribute,
          )
          .push_attribute(
            vk::VertexInputAttributeDescription::build()
              .binding(0)
              .location(1)
              .format(vk::FORMAT_R32G32_UINT)
              .offset(2 * std::mem::size_of::<f32>() as u32)
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

    let pool = descriptor::Pool::new(
      device,
      descriptor::Pool::new_capacity().add(&pipe.dsets[0], 1).add(&pipe.dsets[1], 1),
    );

    Self { pipe, pool }
  }
}


pub struct Text {
  gui: Arc<ImGui>,

  font: FontID,
  pub ub: vk::Buffer,

  vb: vk::Buffer,

  pipe: vk::cmd::commands::BindPipeline,
  ds: vk::cmd::commands::BindDset,
  ds2: vk::cmd::commands::BindDset,
  draw: cmd::commands::DrawVertices,
}

impl Text {
  pub fn new(gui: Arc<ImGui>, _text: &str) -> Self {
    let font = FontID::new("curier", 12);

    let mut vb = vk::NULL_HANDLE;
    let mut ub = vk::NULL_HANDLE;
    vk::mem::Buffer::new(&mut vb)
      .vertex_buffer(3 * std::mem::size_of::<pipe::Vertex>() as vk::DeviceSize)
      .devicelocal(false)
      .new_buffer(&mut ub)
      .uniform_buffer(2 * std::mem::size_of::<f32>() as vk::DeviceSize)
      .devicelocal(false)
      .bind(&mut gui.alloc.clone(), vk::mem::BindType::Block)
      .unwrap();

    {
      let mut map = gui.alloc.get_mapped(vb).unwrap();
      let svb = map.as_slice_mut::<pipe::Vertex>();
      svb[0].pos = cgm::Vector2::new(0, 0);
      svb[0].size = cgm::Vector2::new(50, 50);

      svb[1].pos = cgm::Vector2::new(50, 50);
      svb[1].size = cgm::Vector2::new(50, 50);

      svb[2].pos = cgm::Vector2::new(150, 150);
      svb[2].size = cgm::Vector2::new(50, 50);
    }

    {
      let mut map = gui.alloc.get_mapped(ub).unwrap();
      let data = map.as_slice_mut::<u32>();
      data[0] = 1;
      data[1] = 1;
    }

    let (pipe, ds, ds2) = {
      let mut pp = gui.get_pipe_text();
      let mut p = pp.get_mut();
      let ds = p.pool.new_dset(&p.pipe.dsets[0]).unwrap();
      let ds2 = p.pool.new_dset(&p.pipe.dsets[1]).unwrap();
      (
        cmd::commands::BindPipeline::graphics(p.pipe.handle),
        cmd::commands::BindDset::new(vk::PIPELINE_BIND_POINT_GRAPHICS, p.pipe.layout, 0, ds),
        cmd::commands::BindDset::new(vk::PIPELINE_BIND_POINT_GRAPHICS, p.pipe.layout, 1, ds2),
      )
    };

    pipe::OnResize::write(gui.device, ds.dset).ub_viewport(|b| b.buffer(ub)).update();

    let fnt = gui.get_font(&font);
    pipe::Once::write(gui.device, ds2.dset)
      .tex_sampler(|s| s.set(vk::IMAGE_LAYOUT_SHADER_READ_ONLY_OPTIMAL, fnt.texview, fnt.sampler))
      .update();

    let draw = cmd::commands::Draw::default()
      .push(vb, 0)
      .vertices()
      .instance_count(3)
      .vertex_count(4);

    Text {
      gui: gui.clone(),

      font,

      pipe,
      ds,
      ds2,

      vb,
      ub,

      draw,
    }
  }
}

impl crate::GuiPush for Text {
  fn enqueue(&mut self, cs: cmd::Stream) -> cmd::Stream {
    cs.push(&self.pipe).push(&self.ds).push(&self.ds2).push(&self.draw)
  }
}
