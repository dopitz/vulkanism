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

  pub fn from_vkrect(rect: vk::Rect2D) -> Self {
    Self::from_rect(rect.offset.x, rect.offset.y, rect.extent.width, rect.extent.height)
  }

  pub fn to_vkrect(&self) -> vk::Rect2D {
    vk::Rect2D {
      offset: vk::Offset2D {
        x: self.position.x,
        y: self.position.y,
      },
      extent: vk::Extent2D {
        width: self.size.x,
        height: self.size.y,
      }
    }
  }
}
