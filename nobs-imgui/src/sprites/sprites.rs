use vk;
use vk::cmd::commands::DrawManaged;
use vk::cmd::commands::DrawVertices;
use vk::mem::Handle;
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
  mesh: usize,
  vb_capacity: usize,

  pipe: Pipeline,
}

impl Drop for Sprites {
  fn drop(&mut self) {
    self.gui.get_mem().trash.push_buffer(self.ub);
    self.gui.get_mem().trash.push_buffer(self.vb);
    self
      .gui
      .get_pipe(PipeId::Sprites)
      .dsets
      .as_ref()
      .unwrap()
      .free_dset(self.pipe.bind_ds_instance.dset);
    self.gui.get_drawpass().remove(self.mesh);
  }
}

impl Sprites {
  pub fn new(gui: &ImGui) -> Self {
    let vb = vk::NULL_HANDLE;

    let device = gui.get_device();
    let mut mem = gui.get_mem();

    let mut ub = vk::NULL_HANDLE;
    vk::mem::Buffer::new(&mut ub)
      .uniform_buffer(std::mem::size_of::<UbText>() as vk::DeviceSize)
      .devicelocal(false)
      .bind(&mut mem.alloc, vk::mem::BindType::Block)
      .unwrap();

    let font = gui.get_font();

    let pipe = Pipeline::new(gui.get_pipe(PipeId::Sprites));
    pipe.update_dsets(device, ub, font.texview, font.sampler);

    let mesh = gui.get_drawpass().new_mesh(
      pipe.bind_pipe,
      &[pipe.bind_ds_viewport, pipe.bind_ds_instance],
      DrawManaged::new([(vb, 0)].iter().into(), DrawVertices::with_vertices(4).instance_count(0).into()),
    );

    Sprites {
      device,
      gui: gui.clone(),

      position: Default::default(),

      tex: font.texview,
      sampler: font.sampler,
      vb,
      ub,
      mesh,
      vb_capacity: 0,

      pipe,
    }
  }

  pub fn get_gui(&self) -> ImGui {
    self.gui.clone()
  }

  pub fn get_mesh(&self) -> usize {
    self.mesh
  }

  pub fn position(&mut self, pos: Vec2i) -> &mut Self {
    if self.position != pos {
      self.position = pos;

      let mut map = self.gui.get_mem().alloc.get_mapped(Handle::Buffer(self.ub)).unwrap();
      let data = map.as_slice_mut::<UbText>();
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
    if sprites.len() > self.vb_capacity {
      mem.trash.push_buffer(self.vb);
      self.vb = vk::NULL_HANDLE;

      vk::mem::Buffer::new(&mut self.vb)
        .vertex_buffer((sprites.len() * std::mem::size_of::<Vertex>()) as vk::DeviceSize)
        .devicelocal(false)
        .bind(&mut mem.alloc, vk::mem::BindType::Block)
        .unwrap();

      self.vb_capacity = sprites.len();
    }

    // only copy if not empty
    if !sprites.is_empty() {
      mem.alloc.get_mapped(Handle::Buffer(self.vb)).unwrap().host_to_device_slice(sprites);
    }

    // finally update the buffer and instance count int the mesh
    self.gui.get_drawpass().update_mesh(
      self.mesh,
      None,                                                                             // no pipeline changes
      &[],                                                                              // no dset changes
      &[Some(self.vb)],                                                                 // set the vertex buffer
      Some(DrawVertices::with_vertices(4).instance_count(sprites.len() as u32).into()), // update the nuber of draw instances
    );

    self
  }
}
