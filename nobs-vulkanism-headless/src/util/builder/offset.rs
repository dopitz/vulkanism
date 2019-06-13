use vk;

pub struct Offset2DBuilder {
  offset: vk::Offset2D,
}

vk_builder_into!(vk::Offset2D, Offset2DBuilder, offset);

impl Default for Offset2DBuilder {
  fn default() -> Self {
    Self {
      offset: vk::Offset2D { x: 0, y: 0 },
    }
  }
}

impl Offset2DBuilder {
  pub fn set(mut self, x: i32, y: i32) -> Self {
    self.offset.x = x;
    self.offset.y = y;
    self
  }

  pub fn x(mut self, x: i32) -> Self {
    self.offset.x = x;
    self
  }

  pub fn y(mut self, y: i32) -> Self {
    self.offset.y = y;
    self
  }
}

pub struct Offset3DBuilder {
  offset: vk::Offset3D,
}

vk_builder_into!(vk::Offset3D, Offset3DBuilder, offset);

impl Default for Offset3DBuilder {
  fn default() -> Self {
    Self {
      offset: vk::Offset3D { x: 0, y: 0, z: 0 },
    }
  }
}

impl Offset3DBuilder {
  pub fn set(mut self, x: i32, y: i32, z: i32) -> Self {
    self.offset.x = x;
    self.offset.y = y;
    self.offset.z = z;
    self
  }

  pub fn x(mut self, x: i32) -> Self {
    self.offset.x = x;
    self
  }

  pub fn y(mut self, y: i32) -> Self {
    self.offset.y = y;
    self
  }

  pub fn z(mut self, z: i32) -> Self {
    self.offset.z = z;
    self
  }
}


