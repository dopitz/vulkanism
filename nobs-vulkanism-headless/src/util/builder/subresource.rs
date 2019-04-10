use vk;

pub struct ImageSubresourceBuilder {
  pub subresource: vk::ImageSubresource,
}

vk_builder!(vk::ImageSubresource, ImageSubresourceBuilder, subresource);

impl Default for ImageSubresourceBuilder {
  fn default() -> Self {
    Self {
      subresource: vk::ImageSubresource {
        aspectMask: 0,
        arrayLayer: 0,
        mipLevel: 0,
      },
    }
  }
}

impl ImageSubresourceBuilder {
  pub fn aspect(mut self, mask: vk::ImageAspectFlags) -> Self {
    self.subresource.aspectMask = mask;
    self
  }

  pub fn array_layer(mut self, layer: u32) -> Self {
    self.subresource.arrayLayer = layer;
    self
  }

  pub fn mip_level(mut self, level: u32) -> Self {
    self.subresource.mipLevel = level;
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
