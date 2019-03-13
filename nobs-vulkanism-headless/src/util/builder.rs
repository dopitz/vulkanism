use vk;

pub struct Extent2DBuilder {
  pub extent: vk::Extent2D,
}

vk_builder!(vk::Extent2D, Extent2DBuilder, extent);

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
  pub extent: vk::Extent3D,
}

vk_builder!(vk::Extent3D, Extent3DBuilder, extent);

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

pub struct Offset2DBuilder {
  pub offset: vk::Offset2D,
}

vk_builder!(vk::Offset2D, Offset2DBuilder, offset);

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
  pub offset: vk::Offset3D,
}

vk_builder!(vk::Offset3D, Offset3DBuilder, offset);

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

pub struct ImageSubresourceLayersBuilder {
  pub layers: vk::ImageSubresourceLayers,
}

vk_builder!(vk::ImageSubresourceLayers, ImageSubresourceLayersBuilder, layers);

impl Default for ImageSubresourceLayersBuilder {
  fn default() -> Self {
    Self {
      layers: vk::ImageSubresourceLayers {
        aspectMask: 0,
        baseArrayLayer: 0,
        layerCount: 1,
        mipLevel: 0,
      },
    }
  }
}

impl ImageSubresourceLayersBuilder {
  pub fn aspect(mut self, mask: vk::ImageAspectFlags) -> Self {
    self.layers.aspectMask = mask;
    self
  }

  pub fn base_layer(mut self, layer: u32) -> Self {
    self.layers.baseArrayLayer = layer;
    self
  }

  pub fn layer_count(mut self, count: u32) -> Self {
    self.layers.layerCount = count;
    self
  }

  pub fn mip_level(mut self, level: u32) -> Self {
    self.layers.mipLevel = level;
    self
  }
}
