use vk;
use vk::cmd;
use vk::cmd::commands as cmds;
use vkm::Vec2i;

use super::pipeline::*;
use crate::pipeid::*;
use crate::ImGui;

pub struct Sprites {
  device: vk::Device,
  gui: ImGui,

  position: Vec2i,

  tex: vk::ImageView,
  sampler: vk::Sampler,
  vb: vk::Buffer,
  ub: vk::Buffer,

  pipe: Pipeline,
  draw: cmds::DrawManaged,
}

impl Drop for Sprites {
  fn drop(&mut self) {
    self.gui.get_mem().trash.push(self.ub);
    self.gui.get_mem().trash.push(self.vb);
    self.gui.get_pipe(PipeId::Sprites).dsets.free_dset(self.pipe.bind_ds_instance.dset);
  }
}

impl Sprites {
  pub fn new(gui: &ImGui) -> Self {
    let vb = vk::NULL_HANDLE;

    let device = gui.get_device();
    let mut mem = gui.get_mem();

    let mut ub = vk::NULL_HANDLE;
    vk::mem::Buffer::new(&mut ub)
      .uniform_buffer(std::mem::size_of::<UbViewport>() as vk::DeviceSize)
      .devicelocal(false)
      .bind(&mut mem.alloc, vk::mem::BindType::Block)
      .unwrap();

    let font = gui.get_font();

    let pipe = Pipeline::new(gui.get_pipe(PipeId::Sprites));
    pipe.update_dsets(device, ub, font.texview, font.sampler);

    let draw = Default::default();

    Sprites {
      device,
      gui: gui.clone(),

      position: Default::default(),

      tex: font.texview,
      sampler: font.sampler,
      vb,
      ub,

      pipe,
      draw,
    }
  }

  pub fn position(&mut self, pos: Vec2i) -> &mut Self {
    if self.position != pos {
      self.position = pos;

      let mut map = self.gui.get_mem().alloc.get_mapped(self.ub).unwrap();
      let data = map.as_slice_mut::<UbViewport>();
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
      self.pipe.update_dsets(self.device, self.ub, tex, sampler);
    }
    self
  }

  pub fn sprites(&mut self, sprites: &[Vertex]) -> &mut Self {
    let mut mem = self.gui.get_mem();

    // create new buffer if capacity of cached one is not enough
    if sprites.len() > self.draw.draw.vertices().unwrap().instance_count as usize {
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
      mem.alloc.get_mapped(self.vb).unwrap().host_to_device_slice(sprites);
    }

    // configure the draw call
    self.draw = cmds::DrawManaged::new(
      [(self.vb, 0)].iter().into(),
      cmds::DrawVertices::with_vertices(4).instance_count(sprites.len() as u32).into(),
    );
    self
  }
}

impl cmds::StreamPush for Sprites {
  fn enqueue(&self, cs: cmd::Stream) -> cmd::Stream {
    cs.push(&self.pipe).push(&self.draw)
  }
}
