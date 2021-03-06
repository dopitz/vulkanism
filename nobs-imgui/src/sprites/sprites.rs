use vk;
use vk::cmd::commands::DrawManaged;
use vk::cmd::commands::DrawVertices;
use vk::mem::Handle;
use vkm::Vec2i;

use super::pipeline::*;
use crate::style::Style;
use crate::ImGui;
use vk::pass::MeshId;

/// Stprite rendering
///
/// Renders textured, 2D rectangles. Rectangles are defined in screen space with position (0,0) as the top left corner of the screen and size in pixel.
///
/// Note that you have to set a valid texture and sampler for the sprites with [texture](struct.Sprites.html#method.texture) 
/// otherwise no descriptor set is bound to the sprite rendering pipeline.
pub struct Sprites<S: Style> {
  device: vk::Device,
  gui: ImGui<S>,

  position: Vec2i,

  tex: vk::ImageView,
  sampler: vk::Sampler,
  vb: vk::Buffer,
  ub: vk::Buffer,
  mesh: MeshId,
  vb_capacity: usize,

  pipe: Pipeline,
}

impl<S: Style> Drop for Sprites<S> {
  fn drop(&mut self) {
    self.gui.get_mem().trash.push_buffer(self.ub);
    self.gui.get_mem().trash.push_buffer(self.vb);
    self.gui.get_pipes().sprites.pool.free_dset(self.pipe.bind_ds_instance.dset);
    self.gui.get_drawpass().remove(self.mesh);
  }
}

impl<S: Style> Sprites<S> {
  pub fn new(gui: &ImGui<S>) -> Self {
    let vb = vk::NULL_HANDLE;

    let device = gui.get_device();
    let mut mem = gui.get_mem();

    let mut ub = vk::NULL_HANDLE;
    vk::mem::Buffer::new(&mut ub)
      .uniform_buffer(std::mem::size_of::<UbText>() as vk::DeviceSize)
      .devicelocal(false)
      .bind(&mut mem.alloc, vk::mem::BindType::Block)
      .unwrap();

    let pipe = Pipeline::new_instance(&gui.get_pipes());

    let mesh = gui.get_drawpass().new_mesh(
      pipe.bind_pipe,
      &[pipe.bind_ds_viewport, pipe.bind_ds_instance],
      DrawManaged::new([(vb, 0)].iter().into(), DrawVertices::with_vertices(4).instance_count(0).into()),
    );

    Sprites {
      device,
      gui: gui.clone(),

      position: Default::default(),

      tex: vk::NULL_HANDLE,
      sampler: vk::NULL_HANDLE,
      vb,
      ub,
      mesh,
      vb_capacity: 0,

      pipe,
    }
  }

  pub fn get_gui(&self) -> ImGui<S> {
    self.gui.clone()
  }

  pub fn get_mesh(&self) -> MeshId {
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
