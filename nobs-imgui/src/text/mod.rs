mod pipeline;
use pipeline as pipe;

use crate::cachedpipeline::*;
use crate::font::*;
use crate::rect::Rect;
use crate::window::Window;
use crate::ImGui;

use vk;
use vk::cmd;
use vk::cmd::commands as cmds;

pub struct Text {
  device: vk::Device,
  mem: vk::mem::Mem,

  gui: ImGui,

  ub_viewport: vk::Buffer,
  rect: Rect,

  dirty: bool,
  text: String,
  font: std::sync::Arc<Font>,
  vb: vk::Buffer,
  ub: vk::Buffer,

  pipe: cmds::BindPipeline,
  ds_viewport: cmds::BindDset,
  ds_text: cmds::BindDset,
  draw: cmds::DrawVertices,
}

impl Drop for Text {
  fn drop(&mut self) {
    self.mem.trash.push(self.ub);
    self.mem.trash.push(self.vb);
    let p = &self.gui.get_pipeline::<pipe::Pipeline>().get_cache().pool;
    p.free_dset(self.ds_viewport.dset);
    p.free_dset(self.ds_text.dset);
  }
}

impl Text {
  pub fn new(gui: &ImGui) -> Self {
    let ub_viewport = vk::NULL_HANDLE;
    let vb = vk::NULL_HANDLE;

    let device = gui.get_device();
    let mut mem = gui.get_mem();

    let mut ub = vk::NULL_HANDLE;
    vk::mem::Buffer::new(&mut ub)
      .uniform_buffer(std::mem::size_of::<pipe::Ub>() as vk::DeviceSize)
      .devicelocal(false)
      .bind(&mut mem.alloc, vk::mem::BindType::Block)
      .unwrap();

    println!("{:?}", module_path!());

    let (pipe, ds_viewport, ds_text) = {
      let mut p = gui.get_pipeline::<pipe::Pipeline>();
      (
        cmds::BindPipeline::graphics(p.get_cache().pipe.handle),
        cmds::BindDset::new(
          vk::PIPELINE_BIND_POINT_GRAPHICS,
          p.get_cache().pipe.layout,
          0,
          p.new_ds_viewport().unwrap(),
        ),
        cmds::BindDset::new(
          vk::PIPELINE_BIND_POINT_GRAPHICS,
          p.get_cache().pipe.layout,
          1,
          p.new_ds_text().unwrap(),
        ),
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
      gui: gui.clone(),

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
      draw: cmds::Draw::default().vertices().instance_count(0),
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

      self.draw = cmds::Draw::default()
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

impl cmds::StreamPush for Text {
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
