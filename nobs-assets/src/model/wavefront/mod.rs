mod mtl;
mod obj;

pub use mtl::Mtl;
pub use obj::Obj;
pub use obj::Shape;

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
