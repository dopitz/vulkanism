use super::parse_vec3;
use std::collections::HashMap;
use std::io::Read;
use vkm::Vec3f;

#[derive(Default)]
pub struct Material {
  pub illum: i32,

  pub d: f32,
  pub tr: f32,

  pub ka: Vec3f,
  pub kd: Vec3f,
  pub ks: Vec3f,
  pub ns: f32,
  pub ke: Vec3f,

  pub map_d: String,
  pub map_tr: String,

  pub map_ka: String,
  pub map_kd: String,
  pub map_ks: String,
  pub map_ns: String,
  pub map_ke: String,

  pub map_bump: String,
}

pub struct Mtl {}

impl Mtl {
  pub fn load(file: &str) -> HashMap<String, Material> {
    let mut f = std::fs::File::open(file).unwrap();
    let mut contents = String::new();
    f.read_to_string(&mut contents).unwrap();

    let mut mats: HashMap<String, Material> = HashMap::new();
    let mut m = "".to_string();

    for l in contents.lines() {
      if l.trim().starts_with("newmtl") {
        m = l.split_at(6).1.trim().to_string();
        mats.insert(m.clone(), Default::default());
      } else if !m.is_empty() {
        let (ident, cont) = l.trim().split_at(l.find(char::is_whitespace).unwrap());
        let cont = cont.trim();
        let m = mats.get_mut(&m).unwrap();
        if ident == "illum" {
          m.illum = cont.parse::<i32>().unwrap();
        } else if ident == "d" {
          m.d = cont.parse::<f32>().unwrap();
        } else if ident == "Tr" {
          m.tr = cont.parse::<f32>().unwrap();
        } else if ident == "Ka" {
          m.ka = parse_vec3(cont).unwrap();
        } else if ident == "Kd" {
          m.kd = parse_vec3(cont).unwrap();
        } else if ident == "Ks" {
          m.ks = parse_vec3(cont).unwrap();
        } else if ident == "Ns" {
          m.ns = cont.parse::<f32>().unwrap();
        } else if ident == "Ke" {
          m.ke = parse_vec3(cont).unwrap();
        } else if ident == "map_d" {
          m.map_d = cont.to_string();
        } else if ident == "map_Tr" {
          m.map_tr = cont.to_string();
        } else if ident == "map_Ka" {
          m.map_ka = cont.to_string();
        } else if ident == "map_Kd" {
          m.map_kd = cont.to_string();
        } else if ident == "map_Ks" {
          m.map_ks = cont.to_string();
        } else if ident == "map_Ns" {
          m.map_ns = cont.to_string();
        } else if ident == "map_Ke" {
          m.map_ke = cont.to_string();
        } else if ident == "map_bump" {
          m.map_bump = cont.to_string();
        }
      }
    }

    mats
  }
}
