mod mtl;
mod obj;

pub use mtl::Mtl;
pub use obj::Obj;

use crate::Update;
use vk;
use vk::builder::Buildable;
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

pub struct AssetType {}

impl crate::AssetType for AssetType {
  type Types = Vec<Shape>;
  fn load(id: &str, up: &mut Update) -> Self::Types {
    let obj = Obj::load(id);

    let o = 2 * 3;
    let i = [obj[0].indices[o], obj[0].indices[o + 1], obj[0].indices[o + 2]].iter().map(|i| *i as usize).collect::<Vec<_>>();
    let v = [obj[0].vertices[i[0]], obj[0].vertices[i[1]], obj[0].vertices[i[2]]];
    let n = [obj[0].normals[i[0]], obj[0].normals[i[1]], obj[0].normals[i[2]]];

    let a = v[1] - v[0];
    let b = v[2] - v[0];
    let nt = Vec3f::normalize(Vec3f::cross(a, b));
    let nn = Vec3f::normalize(n[0]);

    let d = Vec3f::dot(nt, nn);

    println!("{:?}", i);
    println!("{:?}", v);
    println!("{:?}", n);
    println!("");

    println!("{:?}", a);
    println!("{:?}", b);
    println!("");

    println!("{:?}", nt);
    println!("{:?}", nn);
    println!("{:?}", d);

    let nn = Vec3f::normalize(n[1]);
    println!("{:?}", nn);
    println!("{:?}", d);

    let nn = Vec3f::normalize(n[2]);
    println!("{:?}", nn);
    println!("{:?}", d);




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
    builder.bind(&mut up.mem.alloc, vk::mem::BindType::Block).unwrap();

    for (s, o) in shapes.iter_mut().zip(obj.iter()) {
      {
        let mut stage = up.get_staging((o.vertices.len() * std::mem::size_of::<vkm::Vec3f>()) as vk::DeviceSize);
        let mut map = stage.map().unwrap();
        map.host_to_device_slice(&o.vertices);
        up.push_buffer((
          stage.copy_into_buffer(s.vertices, 0),
          Some(BufferBarrier::new(s.vertices).to(vk::ACCESS_VERTEX_ATTRIBUTE_READ_BIT)),
        ));
      }

      if !o.normals.is_empty() {
        let mut stage = up.get_staging((o.normals.len() * std::mem::size_of::<vkm::Vec3f>()) as vk::DeviceSize);
        let mut map = stage.map().unwrap();
        map.host_to_device_slice(&o.normals);
        up.push_buffer((
          stage.copy_into_buffer(s.normals, 0),
          Some(BufferBarrier::new(s.normals).to(vk::ACCESS_VERTEX_ATTRIBUTE_READ_BIT)),
        ));
      }

      if !o.uvs.is_empty() {
        let mut stage = up.get_staging((o.uvs.len() * std::mem::size_of::<vkm::Vec2f>()) as vk::DeviceSize);
        let mut map = stage.map().unwrap();
        map.host_to_device_slice(&o.uvs);
        up.push_buffer((
          stage.copy_into_buffer(s.uvs, 0),
          Some(BufferBarrier::new(s.uvs).to(vk::ACCESS_VERTEX_ATTRIBUTE_READ_BIT)),
        ));
      }

      if !o.indices.is_empty() {
        let mut stage = up.get_staging((o.indices.len() * std::mem::size_of::<u32>()) as vk::DeviceSize);
        let mut map = stage.map().unwrap();
        map.host_to_device_slice(&o.indices);
        up.push_buffer((
          stage.copy_into_buffer(s.indices, 0),
          Some(BufferBarrier::new(s.indices).to(vk::ACCESS_VERTEX_ATTRIBUTE_READ_BIT)),
        ));
      }
    }

    shapes
  }
}
