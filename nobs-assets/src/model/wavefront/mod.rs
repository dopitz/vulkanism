mod mtl;
mod obj;

pub use mtl::Mtl;
pub use obj::Obj;

use crate::Update;
use vk;
use vk::cmd::commands::BufferBarrier;
use vkm::Vec2f;
use vkm::Vec3f;

fn parse_vec2(s: &str) -> Result<Vec2f, &'static str> {
  let mut split = s.split_whitespace();
  if split.clone().count() < 2 {
    Err("parse vec2 invalid compontent count")?
  }
  Ok(vec2!(
    split.next().unwrap().parse::<f32>().unwrap(),
    split.next().unwrap().parse::<f32>().unwrap()
  ))
}
fn parse_vec3(s: &str) -> Result<Vec3f, &'static str> {
  let mut split = s.split_whitespace();
  if split.clone().count() < 3 {
    Err("parse vec3 invalid compontent count")?
  }
  Ok(vec3!(
    split.next().unwrap().parse::<f32>().unwrap(),
    split.next().unwrap().parse::<f32>().unwrap(),
    split.next().unwrap().parse::<f32>().unwrap()
  ))
}

pub struct Shape {
  pub name: String,

  pub vertices: vk::Buffer,
  pub normals: vk::Buffer,
  pub uvs: vk::Buffer,

  pub indices: vk::Buffer,
  pub count: u32,
}

pub struct Asset {
  pub shapes: Vec<Shape>,
}

impl crate::Asset for Asset {
  type Id = String;

  fn load(id: &Self::Id, up: &mut Update) -> Self {
    let obj = Obj::load(id);

    let mut shapes = obj
      .iter()
      .map(|s| Shape {
        name: s.name.clone(),
        vertices: vk::NULL_HANDLE,
        normals: vk::NULL_HANDLE,
        uvs: vk::NULL_HANDLE,
        indices: vk::NULL_HANDLE,
        count: 0,
      })
      .collect::<Vec<_>>();

    let mut builder = vk::mem::Resource::new();
    for (s, o) in shapes.iter_mut().zip(obj.iter()) {
      builder = builder
        .new_buffer(&mut s.vertices)
        .vertex_buffer((o.vertices.len() * std::mem::size_of::<vkm::Vec3f>()) as vk::DeviceSize)
        .submit();
      s.count = o.vertices.len() as u32;

      if !o.normals.is_empty() {
        builder = builder
          .new_buffer(&mut s.normals)
          .vertex_buffer((o.normals.len() * std::mem::size_of::<vkm::Vec3f>()) as vk::DeviceSize)
          .submit();
      }

      if !o.uvs.is_empty() {
        builder = builder
          .new_buffer(&mut s.uvs)
          .vertex_buffer((o.uvs.len() * std::mem::size_of::<vkm::Vec2f>()) as vk::DeviceSize)
          .submit();
      }

      if !o.indices.is_empty() {
        builder = builder
          .new_buffer(&mut s.indices)
          .index_buffer((o.indices.len() * std::mem::size_of::<u32>()) as vk::DeviceSize)
          .submit();
        s.count = o.indices.len() as u32;
      }
    }
    builder.bind(&mut up.get_mem().alloc, vk::mem::BindType::Block).unwrap();

    for (s, o) in shapes.iter_mut().zip(obj.iter()) {
      {
        let mut stage = up.get_staging((o.vertices.len() * std::mem::size_of::<vkm::Vec3f>()) as vk::DeviceSize);
        let map = stage.map().unwrap();
        map.host_to_device_slice(&o.vertices);
        up.push_buffer(
          stage.copy_into_buffer(s.vertices, 0),
          Some(BufferBarrier::new(s.vertices).to(vk::ACCESS_VERTEX_ATTRIBUTE_READ_BIT)),
        );
      }

      if !o.normals.is_empty() {
        let mut stage = up.get_staging((o.normals.len() * std::mem::size_of::<vkm::Vec3f>()) as vk::DeviceSize);
        let map = stage.map().unwrap();
        map.host_to_device_slice(&o.normals);
        up.push_buffer(
          stage.copy_into_buffer(s.normals, 0),
          Some(BufferBarrier::new(s.normals).to(vk::ACCESS_VERTEX_ATTRIBUTE_READ_BIT)),
        );
      }

      if !o.uvs.is_empty() {
        let mut stage = up.get_staging((o.uvs.len() * std::mem::size_of::<vkm::Vec2f>()) as vk::DeviceSize);
        let map = stage.map().unwrap();
        map.host_to_device_slice(&o.uvs);
        up.push_buffer(
          stage.copy_into_buffer(s.uvs, 0),
          Some(BufferBarrier::new(s.uvs).to(vk::ACCESS_VERTEX_ATTRIBUTE_READ_BIT)),
        );
      }

      if !o.indices.is_empty() {
        let mut stage = up.get_staging((o.indices.len() * std::mem::size_of::<u32>()) as vk::DeviceSize);
        let map = stage.map().unwrap();
        map.host_to_device_slice(&o.indices);
        up.push_buffer(
          stage.copy_into_buffer(s.indices, 0),
          Some(BufferBarrier::new(s.indices).to(vk::ACCESS_VERTEX_ATTRIBUTE_READ_BIT)),
        );
      }
    }

    Self { shapes }
  }

  fn free(self, up: &mut Update) {
    let mem = up.get_mem();
    for s in self.shapes {
      if s.vertices != vk::NULL_HANDLE {
        mem.trash.push_buffer(s.vertices);
      }
      if s.normals != vk::NULL_HANDLE {
        mem.trash.push_buffer(s.normals);
      }
      if s.uvs != vk::NULL_HANDLE {
        mem.trash.push_buffer(s.uvs);
      }
      if s.indices != vk::NULL_HANDLE {
        mem.trash.push_buffer(s.indices);
      }
    }
  }
}
