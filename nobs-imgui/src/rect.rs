use vkm;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
  pub position: vkm::Vec2i,
  pub size: vkm::Vec2u,
}

impl Default for Rect {
  fn default() -> Self {
    Self::from_rect(0, 0, 0, 0)
  }
}

impl Rect {
  pub fn new(position: vkm::Vec2i, size: vkm::Vec2u) -> Self {
    Self { position, size }
  }

  pub fn from_rect(posx: i32, posy: i32, w: u32, h: u32) -> Self {
    Self::new(vkm::Vec2::new(posx, posy), vkm::Vec2::new(w, h))
  }
}
