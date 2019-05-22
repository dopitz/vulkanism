use vk;

pub struct SamplerBuilder {
  info: vk::SamplerCreateInfo,
}

vk_builder!(vk::SamplerCreateInfo, SamplerBuilder, info);

impl Default for SamplerBuilder {
  fn default() -> Self {
    Self {
      info: vk::SamplerCreateInfo {
      sType: vk::STRUCTURE_TYPE_SAMPLER_CREATE_INFO,
      flags: 0,
      pNext: std::ptr::null(),
      magFilter: vk::FILTER_NEAREST,
      minFilter: vk::FILTER_NEAREST,
      mipmapMode: vk::SAMPLER_MIPMAP_MODE_LINEAR,
      addressModeU: vk::SAMPLER_ADDRESS_MODE_CLAMP_TO_EDGE,
      addressModeV: vk::SAMPLER_ADDRESS_MODE_CLAMP_TO_EDGE,
      addressModeW: vk::SAMPLER_ADDRESS_MODE_CLAMP_TO_EDGE,
      anisotropyEnable: vk::FALSE,
      maxAnisotropy: 1.0,
      borderColor: vk::BORDER_COLOR_INT_OPAQUE_BLACK,
      unnormalizedCoordinates: vk::FALSE,
      compareEnable: vk::FALSE,
      compareOp: vk::COMPARE_OP_ALWAYS,
      mipLodBias: 0.0,
      minLod: 0.0,
      maxLod: 1.0,
      },
    }
  }
}

impl SamplerBuilder {
  pub fn min_filter(mut self, filter: vk::Filter) -> Self {
    self.info.minFilter = filter;
    self
  }

  pub fn mag_filter(mut self, filter: vk::Filter) -> Self {
    self.info.magFilter = filter;
    self
  }

  pub fn mipmap_mode(mut self, m: vk::SamplerMipmapMode) -> Self {
    self.info.mipmapMode = m;
    self
  }

  pub fn address_mode(mut self, u: vk::SamplerAddressMode, v: vk::SamplerAddressMode, w: vk::SamplerAddressMode) -> Self {
    self.info.addressModeU = u;
    self.info.addressModeV = v;
    self.info.addressModeW = w;
    self
  }

  pub fn anisotropy(mut self, enable: vk::Bool32) -> Self {
    self.info.anisotropyEnable = enable;
    self
  }

  pub fn max_anisotropy(mut self, maxaniso: f32) -> Self {
    self.info.maxAnisotropy = maxaniso;
    self
  }

  pub fn border_color(mut self, color: vk::BorderColor) -> Self {
    self.info.borderColor = color;
    self
  }

  pub fn unnormalized_coordinates(mut self, enable: vk::Bool32) -> Self {
    self.info.unnormalizedCoordinates = enable;
    self
  }

  pub fn compare(mut self, enable: vk::Bool32) -> Self {
    self.info.compareEnable = enable;
    self
  }

  pub fn compare_op(mut self, op: vk::CompareOp) -> Self {
    self.info.compareOp = op;
    self
  }

  pub fn mip_lodbias(mut self, bias: f32) -> Self {
    self.info.mipLodBias = bias;
    self
  }

  pub fn min_lod(mut self, lod: f32) -> Self {
    self.info.minLod = lod;
    self
  }

  pub fn max_lod(mut self, lod: f32) -> Self {
    self.info.maxLod = lod;
    self
  }

  pub fn create(&self, device: vk::Device) -> Result<vk::Sampler, vk::Error> {
    let mut sampler = vk::NULL_HANDLE;
    vk_check!(vk::CreateSampler(device, &self.info, std::ptr::null(), &mut sampler))?;
    Ok(sampler)
  }
}


