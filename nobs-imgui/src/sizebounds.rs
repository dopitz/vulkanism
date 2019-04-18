use cgm;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SizeBounds {
  pub position: cgm::Vector2<i32>,
  pub size: cgm::Vector2<u32>,
}

impl Default for SizeBounds {
  fn default() -> Self {
    Self::from_rect(0, 0, 0, 0)
  }
}

impl SizeBounds {
  pub fn new(position: cgm::Vector2<i32>, size: cgm::Vector2<u32>) -> Self {
    Self { position, size }
  }

  pub fn from_rect(posx: i32, posy: i32, w: u32, h: u32) -> Self {
    Self::new(cgm::Vector2::new(posx, posy), cgm::Vector2::new(w, h))
  }
}
