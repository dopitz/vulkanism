use vk;

pub struct Rect2DBuilder {
  pub rect: vk::Rect2D,
}

vk_builder!(vk::Rect2D, Rect2DBuilder, rect);

impl Default for Rect2DBuilder {
  fn default() -> Self {
    Self {
      rect: vk::Rect2D {
        offset: vk::Offset2D { x: 0, y: 0 },
        extent: vk::Extent2D { width: 0, height: 0 },
      },
    }
  }
}

impl Rect2DBuilder {
  pub fn set(mut self, x: i32, y: i32, w: u32, h: u32) -> Self {
    self.rect.extent.width = w;
    self.rect.extent.height = h;
    self
  }

  pub fn x(mut self, x: i32) -> Self {
    self.rect.offset.x = x;
    self
  }

  pub fn y(mut self, y: i32) -> Self {
    self.rect.offset.y = y;
    self
  }

  pub fn offset(self, x: i32, y: i32) -> Self {
    self.x(x).y(y)
  }

  pub fn width(mut self, w: u32) -> Self {
    self.rect.extent.width = w;
    self
  }

  pub fn height(mut self, h: u32) -> Self {
    self.rect.extent.height = h;
    self
  }

  pub fn size(self, w: u32, h: u32) -> Self {
    self.width(w).height(h)
  }
}
