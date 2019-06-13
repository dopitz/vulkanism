use vk;

pub struct Extent2DBuilder {
  extent: vk::Extent2D,
}

vk_builder_into!(vk::Extent2D, Extent2DBuilder, extent);

impl Default for Extent2DBuilder {
  fn default() -> Self {
    Self {
      extent: vk::Extent2D { width: 0, height: 0 },
    }
  }
}

impl Extent2DBuilder {
  pub fn set(mut self, w: u32, h: u32) -> Self {
    self.extent.width = w;
    self.extent.height = h;
    self
  }

  pub fn width(mut self, w: u32) -> Self {
    self.extent.width = w;
    self
  }

  pub fn height(mut self, h: u32) -> Self {
    self.extent.height = h;
    self
  }
}

pub struct Extent3DBuilder {
  extent: vk::Extent3D,
}

vk_builder_into!(vk::Extent3D, Extent3DBuilder, extent);

impl Default for Extent3DBuilder {
  fn default() -> Self {
    Self {
      extent: vk::Extent3D {
        width: 0,
        height: 0,
        depth: 0,
      },
    }
  }
}

impl Extent3DBuilder {
  pub fn set(mut self, w: u32, h: u32, d: u32) -> Self {
    self.extent.width = w;
    self.extent.height = h;
    self.extent.depth = d;
    self
  }

  pub fn width(mut self, w: u32) -> Self {
    self.extent.width = w;
    self
  }

  pub fn height(mut self, h: u32) -> Self {
    self.extent.height = h;
    self
  }

  pub fn depth(mut self, d: u32) -> Self {
    self.extent.depth = d;
    self
  }
}
