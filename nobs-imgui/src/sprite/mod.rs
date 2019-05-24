mod pipeline;
use pipeline as pipe;

use crate::cachedpipeline::*;
use crate::ImGui;

use vk;
use vk::cmd;
use vk::cmd::commands as cmds;
use vkm::Vec2i;

pub use pipe::Vertex;

pub struct Sprites {
  device: vk::Device,
  gui: ImGui,

  position: Vec2i,

  tex: vk::ImageView,
  sampler: vk::Sampler,
  vb: vk::Buffer,
  ub: vk::Buffer,

  pipe: cmds::BindPipeline,
  ds_viewport: cmds::BindDset,
  ds_instance: cmds::BindDset,
  draw: cmds::DrawVertices,
}

impl Drop for Sprites {
  fn drop(&mut self) {
    self.gui.get_mem().trash.push(self.ub);
    self.gui.get_mem().trash.push(self.vb);
    self
      .gui
      .get_pipeline::<pipe::Pipeline>()
      .get_cache()
      .pool
      .free_dset(self.ds_instance.dset);
  }
}

impl Sprites {
  pub fn new(gui: &ImGui) -> Self {
    let vb = vk::NULL_HANDLE;

    let device = gui.get_device();
    let mut mem = gui.get_mem();

    let mut ub = vk::NULL_HANDLE;
    vk::mem::Buffer::new(&mut ub)
      .uniform_buffer(std::mem::size_of::<pipe::Ub>() as vk::DeviceSize)
      .devicelocal(false)
      .bind(&mut mem.alloc, vk::mem::BindType::Block)
      .unwrap();

    let (pipe, ds_viewport, ds_instance) = {
      let mut p = gui.get_pipeline_setup::<pipe::Pipeline, _>(|dsets| {
        pipe::DsViewport::write(device, dsets[0])
          .ub_viewport(|b| b.buffer(gui.get_ub_viewport()))
          .update();
      });
      (
        cmds::BindPipeline::graphics(p.get_cache().pipe.handle),
        cmds::BindDset::new(
          vk::PIPELINE_BIND_POINT_GRAPHICS,
          p.get_cache().pipe.layout,
          0,
          match p.get_cache().shared {
            Some((_, ref ds)) => ds[0],
            None => panic!("should never happen"),
          },
        ),
        cmds::BindDset::new(
          vk::PIPELINE_BIND_POINT_GRAPHICS,
          p.get_cache().pipe.layout,
          1,
          p.new_ds_instance().unwrap(),
        ),
      )
    };

    let font = gui.get_font();
    pipe::DsText::write(device, ds_instance.dset)
      .ub(|b| b.buffer(ub))
      .tex_sampler(|s| s.set(vk::IMAGE_LAYOUT_SHADER_READ_ONLY_OPTIMAL, font.texview, font.sampler))
      .update();

    Sprites {
      device,
      gui: gui.clone(),

      position: Default::default(),

      tex: vk::NULL_HANDLE,
      sampler: vk::NULL_HANDLE,
      vb,
      ub,

      pipe,
      ds_viewport,
      ds_instance,
      draw: cmds::Draw::default().vertices().instance_count(0),
    }
  }

  pub fn position(&mut self, pos: Vec2i) -> &mut Self {
    if self.position != pos {
      self.position = pos;

      let mut map = self.gui.get_mem().alloc.get_mapped(self.ub).unwrap();
      let data = map.as_slice_mut::<pipe::Ub>();
      data[0].offset = pos;
    }
    self
  }
  pub fn get_position(&self) -> Vec2i {
    self.position
  }

  pub fn texture(&mut self, tex: vk::ImageView, sampler: vk::Sampler) -> &mut Self {
    if self.tex != tex || self.sampler != sampler {
      self.tex = tex;
      self.sampler = sampler;
      pipe::DsText::write(self.device, self.ds_instance.dset)
        .tex_sampler(|s| s.set(vk::IMAGE_LAYOUT_SHADER_READ_ONLY_OPTIMAL, tex, sampler))
        .update();
    }
    self
  }

  pub fn sprites(&mut self, sprites: &[Vertex]) -> &mut Self {
    let mut mem = self.gui.get_mem();

    // create new buffer if capacity of cached one is not enough
    if sprites.len() > self.draw.instance_count as usize {
      mem.trash.push(self.vb);
      self.vb = vk::NULL_HANDLE;

      vk::mem::Buffer::new(&mut self.vb)
        .vertex_buffer((sprites.len() * std::mem::size_of::<Vertex>()) as vk::DeviceSize)
        .devicelocal(false)
        .bind(&mut mem.alloc, vk::mem::BindType::Block)
        .unwrap();
    }

    // only copy if not empty
    if !sprites.is_empty() {
      let mut map = mem.alloc.get_mapped(self.vb).unwrap();
      let svb = map.as_slice_mut::<pipe::Vertex>();
      unsafe { std::ptr::copy_nonoverlapping(sprites.as_ptr(), svb.as_mut_ptr(), svb.len()) };
    }

    // configure the draw call
    self.draw = cmds::Draw::default()
      .push(self.vb, 0)
      .vertices()
      .instance_count(sprites.len() as u32)
      .vertex_count(4);
    self
  }
}

impl cmds::StreamPush for Sprites {
  fn enqueue(&self, cs: cmd::Stream) -> cmd::Stream {
    if self.draw.instance_count > 0 {
      cs.push(&self.pipe).push(&self.ds_viewport).push(&self.ds_instance).push(&self.draw)
    } else {
      cs
    }
  }
}
