use std::sync::Arc;

use vk;
use vk::builder::Buildable;
use vk::cmd;
use vk::pipes::descriptor;

use crate::font::FontID;
use crate::window::Window;
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

    dset_name[0] = "DsViewport",
    dset_name[1] = "DsText",
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

  pub fn new_ds_viewport(&mut self) -> Result<vk::DescriptorSet, vk::pipes::Error> {
    self.pool.new_dset(&self.pipe.dsets[0])
  }

  pub fn new_ds_text(&mut self) -> Result<vk::DescriptorSet, vk::pipes::Error> {
    self.pool.new_dset(&self.pipe.dsets[1])
  }
}

pub struct Text {
  font: FontID,
  ub_viewport: vk::Buffer,

  vb: vk::Buffer,

  pipe: vk::cmd::commands::BindPipeline,
  ds_viewport: vk::cmd::commands::BindDset,
  ds_text: vk::cmd::commands::BindDset,
  draw: cmd::commands::DrawVertices,
}

impl Drop for Text {
  fn drop(&mut self) {
    // TODO destroy vb
  }
}

impl Text {
  pub fn new(gui: &ImGui, _text: &str) -> Self {
    let font = FontID::new("curier", 12);
    let ub_viewport = vk::NULL_HANDLE;

    let N = 3usize;

    let mut vb = vk::NULL_HANDLE;
    vk::mem::Buffer::new(&mut vb)
      .vertex_buffer((N * std::mem::size_of::<pipe::Vertex>()) as vk::DeviceSize)
      .devicelocal(false)
      .bind(&mut gui.alloc.clone(), vk::mem::BindType::Block)
      .unwrap();

    {
      let mut map = gui.alloc.get_mapped(vb).unwrap();
      let svb = map.as_slice_mut::<pipe::Vertex>();

      for i in 0..N {
        svb[i].pos = cgm::Vector2::new(50, 50) * i as u32;
        svb[i].size = cgm::Vector2::new(50, 50);
      }
    }

    let (pipe, ds_viewport, ds_text) = {
      let mut p = gui.get_pipe_text();
      (
        cmd::commands::BindPipeline::graphics(p.pipe.handle),
        cmd::commands::BindDset::new(vk::PIPELINE_BIND_POINT_GRAPHICS, p.pipe.layout, 0, p.new_ds_viewport().unwrap()),
        cmd::commands::BindDset::new(vk::PIPELINE_BIND_POINT_GRAPHICS, p.pipe.layout, 1, p.new_ds_text().unwrap()),
      )
    };

    let fnt = gui.get_font(&font);
    pipe::DsText::write(gui.device, ds_text.dset)
      .tex_sampler(|s| s.set(vk::IMAGE_LAYOUT_SHADER_READ_ONLY_OPTIMAL, fnt.texview, fnt.sampler))
      .update();

    let draw = cmd::commands::Draw::default()
      .push(vb, 0)
      .vertices()
      .instance_count(N as u32)
      .vertex_count(4);

    Text {
      font,
      ub_viewport,

      vb,

      pipe,
      ds_viewport,
      ds_text,
      draw,
    }
  }
}

impl vk::cmd::commands::StreamPush for Text {
  fn enqueue(&self, cs: cmd::Stream) -> cmd::Stream {
    cs.push(&self.pipe).push(&self.ds_viewport).push(&self.ds_text).push(&self.draw)
  }
}

impl crate::window::Component for Text {
  fn add_compontent(&mut self, wnd: &Window) {
    if self.ub_viewport != wnd.ub_viewport {
      self.ub_viewport = wnd.ub_viewport;
      pipe::DsViewport::write(wnd.device, self.ds_viewport.dset)
        .ub_viewport(|b| b.buffer(self.ub_viewport))
        .update();
    }
  }
}
