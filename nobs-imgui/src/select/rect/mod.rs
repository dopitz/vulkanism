mod pipeline;

pub use pipeline::Pipeline;
pub use pipeline::Vertex;

use crate::pipeid::*;
use crate::ImGui;
use pipeline::*;
use vk;
use vk::cmd::commands::DrawManaged;
use vk::cmd::commands::DrawVertices;
use vk::mem::Handle;
use vkm::Vec2i;

pub struct Rects {
  device: vk::Device,
  mem: vk::mem::Mem,

  id_offset: u32,

  vb: vk::Buffer,
  ub: vk::Buffer,
  vb_capacity: usize,
}

impl Drop for Rects {
  fn drop(&mut self) {
    self.mem.trash.push_buffer(self.ub);
    self.mem.trash.push_buffer(self.vb);
    //TODO: self.mem.get_pipe(PipeId::Rects).dsets.free_dset(self.pipe.bind_ds_instance.dset);
    //TODO: self.mem.get_meshes().remove(self.mesh);
  }
}

impl Rects {
  pub fn new(device: vk::Device, mut mem: vk::mem::Mem) -> Self {
    let vb = vk::NULL_HANDLE;

    let mut ub = vk::NULL_HANDLE;
    vk::mem::Buffer::new(&mut ub)
      .uniform_buffer(std::mem::size_of::<UbViewport>() as vk::DeviceSize)
      .devicelocal(false)
      .bind(&mut mem.alloc, vk::mem::BindType::Block)
      .unwrap();

    //let pipe = Pipeline::new(gui.get_pipe(PipeId::SelectRects));
    //pipe.update_dsets(device, ub, font.texview, font.sampler);

    Rects {
      device,
      mem,

      position: Default::default(),
      id_offset: 0,

      vb,
      ub,
      vb_capacity: 0,
    }
  }

  pub fn id_offset(&mut self, offset: u32) -> &mut Self {
    if self.id_offset != offset {
      self.id_offset = offset;

      let mut map = self.mem.alloc.get_mapped(Handle::Buffer(self.ub)).unwrap();
      let data = map.as_slice_mut::<UbViewport>();
      data[0].id_offset = offset;
    }
    self
  }
  pub fn get_id_offset(&self) -> u32 {
    self.id_offset
  }

  pub fn rects(&mut self, rects: &[Vertex]) -> &mut Self {
    // create new buffer if capacity of cached one is not enough
    if rects.len() > self.vb_capacity {
      self.mem.trash.push_buffer(self.vb);
      self.vb = vk::NULL_HANDLE;

      vk::mem::Buffer::new(&mut self.vb)
        .vertex_buffer((rects.len() * std::mem::size_of::<Vertex>()) as vk::DeviceSize)
        .devicelocal(false)
        .bind(&mut self.mem.alloc, vk::mem::BindType::Block)
        .unwrap();

      self.vb_capacity = rects.len();
    }

    // only copy if not empty
    if !rects.is_empty() {
      self
        .mem
        .alloc
        .get_mapped(Handle::Buffer(self.vb))
        .unwrap()
        .host_to_device_slice(rects);
    }

    self
  }
}
