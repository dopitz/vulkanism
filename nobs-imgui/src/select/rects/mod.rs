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

pub struct Rects {
  device: vk::Device,
  pass: SelectPass,
  mem: vk::mem::Mem,

  vb: vk::Buffer,
  vb_capacity: usize,
  vb_data: Vec<Vertex>,
  vb_free: BTreeSet<usize>,

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
      pipe,
    }
  }

  pub fn new_rect(&mut self) -> usize {
    match self.vb_free.iter().next().cloned() {
      Some(i) => {
        self.vb_free.remove(&i);
        i
      }
      None => {
        self.vb_data.push(Default::default());
        self.vb_data.len() - 1
      }
    }
  }
  pub fn remove(&mut self, i: usize) {
    self.vb_free.insert(i);
  }

  pub fn get(&self, i: usize) -> &Vertex {
    &self.vb_data[i]
  }
  pub fn get_mut(&mut self, i: usize) -> &mut Vertex {
    &mut self.vb_data[i]
  }

  pub fn update(&mut self) {
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
  }
}
