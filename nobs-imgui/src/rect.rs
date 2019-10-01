use vk;
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

  pub fn union(a: Self, b: Self) -> Self {
    let p = vkm::Vec2::min(a.position, b.position);
    let q = vkm::Vec2::max(a.position + a.size.into(), b.position + b.size.into());
    Self::new(p, (q - p).into())
  }
}

impl From<vk::Rect2D> for Rect {
  fn from(r: vk::Rect2D) -> Self {
    Self::from_rect(r.offset.x, r.offset.y, r.extent.width, r.extent.height)
  }
}

impl Into<vk::Rect2D> for Rect {
  fn into(self) -> vk::Rect2D {
    vk::Rect2D {
      offset: vk::Offset2D {
        x: self.position.x,
        y: self.position.y,
      },
      extent: vk::Extent2D {
        width: self.size.x,
        height: self.size.y,
      },
    }
  }
}
