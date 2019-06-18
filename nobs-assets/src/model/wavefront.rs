use std::collections::HashMap;
use std::convert::TryFrom;
use std::convert::TryInto;
use std::io::Read;
use vk;
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

enum Token {
  Vert(Vec3f),
  Norm(Vec3f),
  UV(Vec2f),
  Group(String),
  Face([Option<FaceIndex>; 4]),
  Smooth(bool),
  Comment,
}

impl TryFrom<&str> for Token {
  type Error = &'static str;

  fn try_from(line: &str) -> Result<Self, Self::Error> {
    let line = line.trim_start();
    if line.starts_with("v ") {
      Ok(Token::Vert(parse_vec3(line.split_at(1).1)?))
    } else if line.starts_with("vn ") {
      Ok(Token::Norm(parse_vec3(line.split_at(2).1)?))
    } else if line.starts_with("vt ") {
      Ok(Token::UV(parse_vec2(line.split_at(2).1)?))
    } else if line.starts_with("g ") {
      Ok(Token::Group(line.split_at(1).1.trim().to_string()))
    } else if line.starts_with("f ") {
      let mut face: [Option<FaceIndex>; 4] = [None, None, None, None];
      for (i, idx) in line.split_whitespace().skip(1).enumerate() {
        let mut fi = FaceIndex::default();
        let mut split = idx.split('/');

        let mut parse_index = || split.next().unwrap().parse::<i32>().unwrap();

        fi.v = parse_index();
        fi.vt = parse_index();
        fi.vn = parse_index();
        face[i] = Some(fi);
      }
      Ok(Token::Face(face))
    } else if line.starts_with("s ") {
      Ok(Token::Smooth(line.split_at(1).1.trim().starts_with("1")))
    } else if line.starts_with("#") || line.is_empty() {
      Ok(Token::Comment)
    } else {
      Err("unrecognized token")
    }
  }
}

#[derive(Default, Hash, PartialEq, Eq, Clone, Copy)]
struct FaceIndex {
  v: i32,
  vt: i32,
  vn: i32,
}

struct Group {
  name: String,
  faces: Vec<[Option<FaceIndex>; 4]>,
}

#[derive(Default)]
struct Shape {
  name: String,
  vertices: Vec<Vec3f>,
  normals: Vec<Vec3f>,
  uvs: Vec<Vec2f>,

  indices: Vec<u16>,
}

pub struct Wavefront {}

impl Wavefront {
  pub fn load(file: &str) {
    let mut f = std::fs::File::open(file).unwrap();
    let mut contents = String::new();
    f.read_to_string(&mut contents).unwrap();

    let mut vertices: Vec<Vec3f> = Default::default();
    let mut normals: Vec<Vec3f> = Default::default();
    let mut uvs: Vec<Vec2f> = Default::default();
    let mut groups: Vec<Group> = Default::default();

    let mut group: Option<&mut Group> = None;

    // parse file
    for l in contents.lines() {
      let tok = match l.try_into() {
        Err(e) => panic!("{}:  {}", e, l),
        Ok(t) => t,
      };
      match tok {
        Token::Vert(v) => vertices.push(v),
        Token::Norm(n) => normals.push(n),
        Token::UV(uv) => uvs.push(uv),
        Token::Face(f) => group.as_mut().unwrap().faces.push(f),
        Token::Group(g) => {
          groups.push(Group {
            name: g,
            faces: Default::default(),
          });
          group = groups.last_mut();
        }
        _ => (),
      }
    }

    // fix face indices
    for g in groups.iter_mut() {
      for f in g.faces.iter_mut() {
        for fi in f.iter_mut().filter_map(|fi| fi.as_mut()) {
          // relative indices are negative
          if fi.v < 0 {
            fi.v += vertices.len() as i32 + 1;
          }
          if fi.vt < 0 {
            fi.vt += uvs.len() as i32 + 1;
          }
          if fi.vn < 0 {
            fi.vn += normals.len() as i32 + 1;
          }

          // obj indices are 1 based, so we sutract 1 to get the correct indicex into our vectors
          fi.v -= 1;
          fi.vt -= 1;
          fi.vn -= 1;
        }
      }
    }

    // convert into shapes
    let mut shapes = Vec::with_capacity(groups.len());
    let mut face_indices: HashMap<FaceIndex, u16> = Default::default();

    for g in groups.iter() {
      face_indices.clear();
      let mut shape: Shape = Default::default();
      shape.name = g.name.clone();

      let mut indices = [0, 0, 0, 0];
      for f in g.faces.iter() {
        for (i, fi) in f.iter().filter_map(|fi| *fi).enumerate() {
          indices[i] = *face_indices.entry(fi).or_insert_with(|| {
            let i = shape.vertices.len();
            shape.vertices.push(vertices[fi.v as usize]);
            shape.normals.push(normals[fi.vn as usize]);
            shape.uvs.push(uvs[fi.vt as usize]);
            i as u16
          });
        }

        if f[3].is_some() {
          shape.indices.push(indices[0]);
          shape.indices.push(indices[1]);
          shape.indices.push(indices[2]);

          shape.indices.push(indices[0]);
          shape.indices.push(indices[2]);
          shape.indices.push(indices[3]);
        } else {
          shape.indices.push(indices[0]);
          shape.indices.push(indices[1]);
          shape.indices.push(indices[2]);
        }
      }

      shapes.push(shape);
    }
  }
}
