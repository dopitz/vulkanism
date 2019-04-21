use vk;
use vk::builder::Buildable;
use vk::cmd;
use vk::pipes::descriptor;

use crate::font::FontID;
use crate::sizebounds::SizeBounds;
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
    pub tex_bl: cgm::Vector2<f32>,
    pub tex_tr: cgm::Vector2<f32>,
  }

  #[repr(C)]
  pub struct Ub {
    pub offset: cgm::Vector2<i32>,
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
  alloc: vk::mem::Allocator,
  unused: vk::mem::UnusedResources,

  ub_viewport: vk::Buffer,
  bounds: SizeBounds,

  dirty: bool,
  text: String,
  font: FontID,
  vb: vk::Buffer,
  ub: vk::Buffer,

  pipe: vk::cmd::commands::BindPipeline,
  ds_viewport: vk::cmd::commands::BindDset,
  ds_text: vk::cmd::commands::BindDset,
  draw: cmd::commands::DrawVertices,
}

impl Drop for Text {
  fn drop(&mut self) {
    self.alloc.destroy_many(&[self.ub, self.vb]);
  }
}

impl Text {
  pub fn new(gui: &ImGui) -> Self {
    let ub_viewport = vk::NULL_HANDLE;

    let font = FontID::new("curier", 12);
    let vb = vk::NULL_HANDLE;

    let mut ub = vk::NULL_HANDLE;
    vk::mem::Buffer::new(&mut ub)
      .uniform_buffer(std::mem::size_of::<pipe::Ub>() as vk::DeviceSize)
      .devicelocal(false)
      .bind(&mut gui.alloc.clone(), vk::mem::BindType::Block)
      .unwrap();

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
      .ub(|b| b.buffer(ub))
      .tex_sampler(|s| s.set(vk::IMAGE_LAYOUT_SHADER_READ_ONLY_OPTIMAL, fnt.texview, fnt.sampler))
      .update();

    Text {
      alloc: gui.alloc.clone(),
      unused: gui.unused.clone(),

      ub_viewport,
      bounds: Default::default(),

      dirty: true,
      text: Default::default(),
      font,
      vb,
      ub,

      pipe,
      ds_viewport,
      ds_text,
      draw: cmd::commands::Draw::default().vertices().instance_count(0),
    }
  }

  pub fn font(&mut self, font: FontID) -> &mut Self {
    if self.font != font {
      self.font = font;
      self.dirty = true;
    }
    self
  }

  pub fn text(&mut self, text: &str) -> &mut Self {
    if self.text != text {
      self.text = text.to_owned();
      self.dirty = true;
    }
    self
  }

  fn update_vb(&mut self) {
    if !self.dirty {
      return;
    }

    // create new buffer
    if self.text.len() > self.draw.instance_count as usize {
      self.unused.push(self.vb);
      self.vb = vk::NULL_HANDLE;

      vk::mem::Buffer::new(&mut self.vb)
        .vertex_buffer((self.text.len() * std::mem::size_of::<pipe::Vertex>()) as vk::DeviceSize)
        .devicelocal(false)
        .bind(&mut self.alloc.clone(), vk::mem::BindType::Block)
        .unwrap();

      self.draw = cmd::commands::Draw::default()
        .push(self.vb, 0)
        .vertices()
        .instance_count(self.text.len() as u32)
        .vertex_count(4);
    }

    let mut map = self.alloc.get_mapped(self.vb).unwrap();
    let svb = map.as_slice_mut::<pipe::Vertex>();

    for i in 0..self.text.len() {
      svb[i].pos = cgm::Vector2::new(50, 50) * i as u32;
      svb[i].size = cgm::Vector2::new(50, 50);
      svb[i].tex_bl = cgm::Vector2::new(0.0, 1.0);
      svb[i].tex_tr = cgm::Vector2::new(1.0, 0.0);
    }

    self.dirty = false;
  }
}

impl vk::cmd::commands::StreamPush for Text {
  fn enqueue(&self, cs: cmd::Stream) -> cmd::Stream {
    cs.push(&self.pipe).push(&self.ds_viewport).push(&self.ds_text).push(&self.draw)
  }
}

impl crate::window::Component for Text {
  fn add_compontent(&mut self, wnd: &mut Window) {
    if self.ub_viewport != wnd.ub_viewport {
      self.ub_viewport = wnd.ub_viewport;
      pipe::DsViewport::write(wnd.device, self.ds_viewport.dset)
        .ub_viewport(|b| b.buffer(self.ub_viewport))
        .update();
    }

    let bounds = wnd.get_next_bounds();
    if self.bounds != bounds {
      self.bounds = bounds;

      let mut map = self.alloc.get_mapped(self.ub).unwrap();
      let data = map.as_slice_mut::<pipe::Ub>();
      data[0].offset = bounds.position;
    }

    self.update_vb();
  }
}
