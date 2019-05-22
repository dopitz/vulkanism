use vk;
use vk::builder::Buildable;
use vk::cmd;
use vk::pipes::descriptor;

use crate::font::*;
use crate::rect::Rect;
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
  device: vk::Device,
  mem: vk::mem::Mem,

  ub_viewport: vk::Buffer,
  rect: Rect,

  dirty: bool,
  text: String,
  font: std::sync::Arc<Font>,
  vb: vk::Buffer,
  ub: vk::Buffer,

  pipe: vk::cmd::commands::BindPipeline,
  ds_viewport: vk::cmd::commands::BindDset,
  ds_text: vk::cmd::commands::BindDset,
  draw: cmd::commands::DrawVertices,
}

impl Drop for Text {
  fn drop(&mut self) {
    self.mem.trash.push(self.ub);
    self.mem.trash.push(self.vb);
    //let p = self.gui.get_pipe_text().pool;
    //p.free_dset(self.ds_viewport);
    //p.free_dset(self.ds_text);
  }
}

impl Text {
  pub fn new(gui: &ImGui) -> Self {
    let ub_viewport = vk::NULL_HANDLE;
    let vb = vk::NULL_HANDLE;

    let device = gui.device;
    let mut mem = gui.mem.clone();

    let mut ub = vk::NULL_HANDLE;
    vk::mem::Buffer::new(&mut ub)
      .uniform_buffer(std::mem::size_of::<pipe::Ub>() as vk::DeviceSize)
      .devicelocal(false)
      .bind(&mut mem.alloc, vk::mem::BindType::Block)
      .unwrap();

    println!("{:?}", module_path!());

    let (pipe, ds_viewport, ds_text) = {
      let mut p = gui.get_pipe_text();
      (
        cmd::commands::BindPipeline::graphics(p.pipe.handle),
        cmd::commands::BindDset::new(vk::PIPELINE_BIND_POINT_GRAPHICS, p.pipe.layout, 0, p.new_ds_viewport().unwrap()),
        cmd::commands::BindDset::new(vk::PIPELINE_BIND_POINT_GRAPHICS, p.pipe.layout, 1, p.new_ds_text().unwrap()),
      )
    };

    let font = gui.get_font();
    pipe::DsText::write(device, ds_text.dset)
      .ub(|b| b.buffer(ub))
      .tex_sampler(|s| s.set(vk::IMAGE_LAYOUT_SHADER_READ_ONLY_OPTIMAL, font.texview, font.sampler))
      .update();

    Text {
      device,
      mem,

      ub_viewport,
      rect: Default::default(),

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

  pub fn font(&mut self, font: std::sync::Arc<Font>) -> &mut Self {
    if !std::sync::Arc::ptr_eq(&self.font, &font) {
      self.font = font;
      pipe::DsText::write(self.device, self.ds_text.dset)
        .tex_sampler(|s| s.set(vk::IMAGE_LAYOUT_SHADER_READ_ONLY_OPTIMAL, self.font.texview, self.font.sampler))
        .update();
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
      self.mem.trash.push(self.vb);
      self.vb = vk::NULL_HANDLE;

      vk::mem::Buffer::new(&mut self.vb)
        .vertex_buffer((self.text.len() * std::mem::size_of::<pipe::Vertex>()) as vk::DeviceSize)
        .devicelocal(false)
        .bind(&mut self.mem.alloc, vk::mem::BindType::Block)
        .unwrap();

      self.draw = cmd::commands::Draw::default()
        .push(self.vb, 0)
        .vertices()
        .instance_count(self.text.len() as u32)
        .vertex_count(4);
    }

    let mut map = self.mem.alloc.get_mapped(self.vb).unwrap();
    let svb = map.as_slice_mut::<pipe::Vertex>();

    //TypeSet::new(&*self.font).offset(vec2!(250.0)).size(150.0).compute(&self.text, svb);
    TypeSet::new(&*self.font).offset(vec2!(250.0)).size(20.0).compute(&self.text, svb);

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

    let rect = wnd.get_next_bounds();
    if self.rect != rect {
      self.rect = rect;

      let mut map = self.mem.alloc.get_mapped(self.ub).unwrap();
      let data = map.as_slice_mut::<pipe::Ub>();
      data[0].offset = rect.position;
    }

    self.update_vb();
  }
}
