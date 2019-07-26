mod pipeline;

pub use pipeline::Pipeline;
pub use pipeline::Vertex;

use crate::pipeid::*;
use crate::select::SelectPass;
use crate::ImGui;
use pipeline::*;
use std::collections::BTreeSet;
use vk;
use vk::cmd::commands::DrawManaged;
use vk::cmd::commands::DrawVertices;
use vk::mem::Handle;
use vkm::Vec2i;
use vkm::Vec2u;

pub struct Rects {
  device: vk::Device,
  pass: SelectPass,
  mem: vk::mem::Mem,

  vb: vk::Buffer,
  vb_capacity: usize,
  vb_data: Vec<Vertex>,
  vb_free: BTreeSet<usize>,
  vb_dirty: bool,
  meshes: Vec<usize>,

  pipe: Pipeline,
}

impl Drop for Rects {
  fn drop(&mut self) {
    self.mem.trash.push_buffer(self.vb);
  }
}

impl Rects {
  pub fn new(device: vk::Device, pass: SelectPass, pipe: &vk::pipes::CachedPipeline, mem: vk::mem::Mem) -> Self {
    let vb = vk::NULL_HANDLE;

    let pipe = Pipeline::new(pipe);

    Rects {
      device,
      pass,
      mem,

      vb,
      vb_capacity: 0,
      vb_data: Default::default(),
      vb_free: Default::default(),
      vb_dirty: false,
      meshes: Default::default(),
      pipe,
    }
  }

  pub fn new_rect(&mut self, pos: Vec2i, size: Vec2u) -> usize {
    let rect = match self.vb_free.iter().next().cloned() {
      Some(i) => {
        self.vb_free.remove(&i);
        i
      }
      None => {
        self.meshes.push(0);
        self.vb_data.push(Default::default());
        self.vb_data.len() - 1
      }
    };
    self.vb_data[rect].id = self.pass.new_id();
    self.meshes[rect] = self.pass.new_mesh(
      self.pipe.bind_pipe,
      &[self.pipe.bind_ds_viewport],
      DrawManaged::new(
        [(self.vb, 0)].iter().into(),
        DrawVertices::with_vertices(4).instance_count(1).first_instance(rect as u32).into(),
      ),
    );
    self.update_rect(rect, pos, size);
    rect
  }

  pub fn update_rect(&mut self, i: usize, pos: Vec2i, size: Vec2u) {
    self.vb_data[i].pos = pos.into();
    self.vb_data[i].size = size.into();
    self.vb_dirty = true;
  }

  pub fn remove(&mut self, i: usize) {
    self.pass.remove_mesh(self.meshes[i]);
    self.pass.remove_id(self.vb_data[i].id);
    self.vb_free.insert(i);
  }

  pub fn get(&self, i: usize) -> &Vertex {
    &self.vb_data[i]
  }

  pub fn get_mesh(&self, i: usize) -> usize {
    self.meshes[i]
  }

  pub fn update(&mut self) {
    if self.vb_dirty {
      // create new buffer if capacity of cached one is not enough
      if self.vb_data.len() > self.vb_capacity {
        self.mem.trash.push_buffer(self.vb);
        self.vb = vk::NULL_HANDLE;

        vk::mem::Buffer::new(&mut self.vb)
          .vertex_buffer((self.vb_data.len() * std::mem::size_of::<Vertex>()) as vk::DeviceSize)
          .devicelocal(false)
          .bind(&mut self.mem.alloc, vk::mem::BindType::Block)
          .unwrap();

        self.vb_capacity = self.vb_data.len();

        // update the vertex buffer in the draw meshes of the select pass
        for m in self.meshes.iter() {
          let mesh = self.pass.update_mesh(*m, None, &[], &[Some(self.vb)], None);
        }
      }

      // only copy if not empty
      if !self.vb_data.is_empty() {
        self
          .mem
          .alloc
          .get_mapped(Handle::Buffer(self.vb))
          .unwrap()
          .host_to_device_slice(&self.vb_data);
      }
      self.vb_dirty = false;
    }
  }
}
