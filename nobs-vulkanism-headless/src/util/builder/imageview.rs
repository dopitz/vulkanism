use vk;

pub struct ImageViewBuilder {
  info: vk::ImageViewCreateInfo,
}

vk_builder!(vk::ImageViewCreateInfo, ImageViewBuilder, info);

impl Default for ImageViewBuilder {
  fn default() -> Self {
    Self {
      info: vk::ImageViewCreateInfo {
        sType: vk::STRUCTURE_TYPE_IMAGE_VIEW_CREATE_INFO,
        pNext: std::ptr::null(),
        flags: 0,
        image: vk::NULL_HANDLE,
        viewType: vk::IMAGE_VIEW_TYPE_2D,
        format: vk::FORMAT_UNDEFINED,
        components: vk::ComponentMapping {
          r: vk::COMPONENT_SWIZZLE_IDENTITY,
          g: vk::COMPONENT_SWIZZLE_IDENTITY,
          b: vk::COMPONENT_SWIZZLE_IDENTITY,
          a: vk::COMPONENT_SWIZZLE_IDENTITY,
        },
        subresourceRange: vk::ImageSubresourceRange {
          aspectMask: 0,
          baseMipLevel: 0,
          levelCount: 1,
          baseArrayLayer: 0,
          layerCount: 1,
        },
      },
    }
  }
}

impl ImageViewBuilder {
  pub fn texture2d(self, image: vk::Image, format: vk::Format) -> Self {
    self.image(image).format(format).aspect(vk::IMAGE_ASPECT_COLOR_BIT)
  }

  pub fn image(mut self, image: vk::Image) -> Self {
    self.info.image = image;
    self
  }

  pub fn view_type(mut self, ty: vk::ImageViewType) -> Self {
    self.info.viewType = ty;
    self
  }

  pub fn format(mut self, format: vk::Format) -> Self {
    self.info.format = format;
    self
  }

  pub fn compontents(mut self, components: vk::ComponentMapping) -> Self {
    self.info.components = components;
    self
  }

  pub fn subresource(mut self, subresource: vk::ImageSubresourceRange) -> Self {
    self.info.subresourceRange = subresource;
    self
  }
  pub fn aspect(mut self, aspect: vk::ImageAspectFlags) -> Self {
    self.info.subresourceRange.aspectMask = aspect;
    self
  }
  pub fn mip_levels(mut self, base_level: u32, count: u32) -> Self {
    self.info.subresourceRange.baseMipLevel = base_level;
    self.info.subresourceRange.levelCount = count;
    self
  }
  pub fn array_layers(mut self, base_layer: u32, count: u32) -> Self {
    self.info.subresourceRange.baseArrayLayer = base_layer;
    self.info.subresourceRange.layerCount = count;
    self
  }

  pub fn create(&self, device: vk::Device) -> Result<vk::ImageView, vk::Error> {
    let mut view = vk::NULL_HANDLE;
    vk_check!(vk::CreateImageView(device, &self.info, std::ptr::null(), &mut view))?;
    Ok(view)
  }
}

