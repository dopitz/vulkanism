use vk;

pub struct Swapchain {
  device: vk::Device,
  ext: vk::DeviceExtensions,
  pub handle: vk::SwapchainKHR,
  pub images: Vec<vk::Image>,
  pub views: Vec<vk::ImageView>,
}

impl Drop for Swapchain {
  fn drop(&mut self) {
    for v in self.views.iter() {
      vk::DestroyImageView(self.device, *v, std::ptr::null());
    }
    self.ext.DestroySwapchainKHR(self.device, self.handle, std::ptr::null());
  }
}

impl Swapchain {
  pub fn build(inst: vk::Instance, pdevice: vk::PhysicalDevice, device: vk::Device, surface: vk::SurfaceKHR) -> Builder {
    Builder::init(inst, pdevice, device, surface)
  }

  //swapchain::next_img swapchain::acquire_next_image() {
  //  uint32_t index;
  //  LOGVK(vkAcquireNextImageKHR(_device->get_device_handle(), _handle, std::numeric_limits<uint64_t>::max(), _signal_next_image, VK_NULL_HANDLE, &index));
  //  return {_signal_next_image, index};
  //}
  //swapchain::command_blit swapchain::blit(h_image im, uint32_t sc_image_index, VkImageLayout final_layout, VkAccessFlags access) {
  //  auto& img = *im;
  //  command_blit blit;
  //  blit.region.srcSubresource.aspectMask = VK_IMAGE_ASPECT_COLOR_BIT;
  //  blit.region.srcSubresource.mipLevel = 0;
  //  blit.region.srcSubresource.baseArrayLayer = 0;
  //  blit.region.srcSubresource.layerCount = 1;
  //  blit.region.srcOffsets[0] = {0, 0, 0};
  //  blit.region.srcOffsets[1] = {static_cast<int32_t>(img.get_dim().x), static_cast<int32_t>(img.get_dim().y), 1};
  //  blit.region.dstSubresource.aspectMask = VK_IMAGE_ASPECT_COLOR_BIT;
  //  blit.region.dstSubresource.mipLevel = 0;
  //  blit.region.dstSubresource.baseArrayLayer = 0;
  //  blit.region.dstSubresource.layerCount = 1;
  //  blit.region.dstOffsets[0] = {0, 0, 0};
  //  blit.region.dstOffsets[1] = {_image_size.x, _image_size.y, 1};

  //  blit.src = img.get_handle();
  //  blit.src_final_layout = final_layout;
  //  blit.src_access = access;
  //  blit.dst = _images[sc_image_index];
  //  blit.filter = VK_FILTER_LINEAR;

  //  return blit;
  //}
  //swapchain::command_blit swapchain::blit(next_img& next, h_image im, VkImageLayout final_layout, VkPipelineStageFlags stage) {
  //  next = acquire_next_image();
  //  return blit(im, next.index, final_layout, stage);
  //}
  //swapchain::command_resolve swapchain::resolve(h_image im, uint32_t sc_image_index, VkImageLayout final_layout, VkAccessFlags access) {
  //  auto& img = *im;
  //  command_resolve resolve;
  //  resolve.region.srcSubresource.aspectMask = VK_IMAGE_ASPECT_COLOR_BIT;
  //  resolve.region.srcSubresource.mipLevel = 0;
  //  resolve.region.srcSubresource.baseArrayLayer = 0;
  //  resolve.region.srcSubresource.layerCount = 1;
  //  resolve.region.srcOffset = {0, 0, 0};
  //  resolve.region.dstSubresource.aspectMask = VK_IMAGE_ASPECT_COLOR_BIT;
  //  resolve.region.dstSubresource.mipLevel = 0;
  //  resolve.region.dstSubresource.baseArrayLayer = 0;
  //  resolve.region.dstSubresource.layerCount = 1;
  //  resolve.region.dstOffset = {0, 0, 0};
  //  resolve.region.extent = {img.get_dim().x, img.get_dim().y, 1};

  //  resolve.src = img.get_handle();
  //  resolve.src_final_layout = final_layout;
  //  resolve.src_access = access;
  //  resolve.dst = _images[sc_image_index];

  //  return resolve;
  //}
  //swapchain::command_resolve swapchain::resolve(next_img& next, h_image im, VkImageLayout final_layout, VkPipelineStageFlags stage) {
  //  next = acquire_next_image();
  //  return resolve(im, next.index, final_layout, stage);
  //}
  //swapchain::command_blit_resolve swapchain::blit(
  //    VkSampleCountFlagBits sample_count, h_image im, uint32_t sc_image_index, VkImageLayout final_layout, VkAccessFlags access) {
  //  command_blit_resolve br;
  //  if (sample_count == VK_SAMPLE_COUNT_1_BIT)
  //    br.cmd.blit = blit(im, sc_image_index, final_layout, access);
  //  else
  //    br.cmd.resolve = resolve(im, sc_image_index, final_layout, access);
  //  br.sample_count = sample_count;
  //  return br;
  //}
  //swapchain::command_blit_resolve swapchain::blit(
  //    VkSampleCountFlagBits sample_count, next_img& next, h_image im, VkImageLayout final_layout, VkAccessFlags access) {
  //  next = acquire_next_image();
  //  return blit(sample_count, im, next.index, final_layout, access);
  //}

  pub fn present(&self, q: vk::Queue, sc_image_index: u32, wait_for: &[vk::Semaphore]) {
    let present_info = vk::PresentInfoKHR {
      sType: vk::STRUCTURE_TYPE_PRESENT_INFO_KHR,
      pNext: std::ptr::null(),
      waitSemaphoreCount: wait_for.len() as u32,
      pWaitSemaphores: wait_for.as_ptr(),
      swapchainCount: 1,
      pSwapchains: &self.handle,
      pImageIndices: &sc_image_index,
      pResults: std::ptr::null_mut(),
    };

    self.ext.QueuePresentKHR(q, &present_info);
  }

  //VkSemaphore swapchain::get_signal_next_image() {
  //  return _signal_next_image;
  //}

  //float swapchain::get_aspect() const {
  //  return _image_size.x / (float)_image_size.y;
  //}
  //math::vec2i swapchain::get_size() const {
  //  return _image_size;
  //}
  //VkExtent2D swapchain::get_extent() const {
  //  return {uint32_t(_image_size.x), uint32_t(_image_size.y)};
  //}
  //VkFormat swapchain::get_format() const {
  //  return _image_format;
  //}
  //VkColorSpaceKHR swapchain::get_colorspace() const {
  //  return _image_colorspace;
  //}
  //VkPresentModeKHR swapchain::get_presentmode() const {
  //  return _presentmode;
  //}
}

pub struct Builder {
  device: vk::Device,
  capabilities: vk::SurfaceCapabilitiesKHR,
  info: vk::SwapchainCreateInfoKHR,
}

impl Builder {
  fn get_default_format(
    ext: &vk::InstanceExtensions,
    pdevice: vk::PhysicalDevice,
    surface: vk::SurfaceKHR,
  ) -> (vk::Format, vk::ColorSpaceKHR) {
    let mut format_count = 0;
    ext.GetPhysicalDeviceSurfaceFormatsKHR(pdevice, surface, &mut format_count, std::ptr::null_mut());
    let mut formats = Vec::with_capacity(format_count as usize);
    ext.GetPhysicalDeviceSurfaceFormatsKHR(pdevice, surface, &mut format_count, formats.as_mut_ptr());
    unsafe {
      formats.set_len(format_count as usize);
    }

    debug_assert!(!formats.is_empty());

    if formats.len() == 1 && formats[0].format == vk::FORMAT_UNDEFINED {
      return (vk::FORMAT_B8G8R8A8_UNORM, vk::COLOR_SPACE_SRGB_NONLINEAR_KHR);
    }

    match formats
      .iter()
      .find(|f| f.format == vk::FORMAT_B8G8R8A8_UNORM && f.colorSpace == vk::COLOR_SPACE_SRGB_NONLINEAR_KHR)
    {
      Some(f) => (f.format, f.colorSpace),
      None => (formats[0].format, formats[0].colorSpace),
    }
  }
  fn get_default_presentmode(ext: &vk::InstanceExtensions, pdevice: vk::PhysicalDevice, surface: vk::SurfaceKHR) -> vk::PresentModeKHR {
    let mut mode_count = 0;
    ext.GetPhysicalDeviceSurfacePresentModesKHR(pdevice, surface, &mut mode_count, std::ptr::null_mut());
    let mut modes = Vec::with_capacity(mode_count as usize);
    ext.GetPhysicalDeviceSurfacePresentModesKHR(pdevice, surface, &mut mode_count, modes.as_mut_ptr());
    unsafe {
      modes.set_len(mode_count as usize);
    }

    match modes.iter().find(|m| **m == vk::PRESENT_MODE_MAILBOX_KHR) {
      Some(m) => *m,
      None => match modes.iter().find(|m| **m == vk::PRESENT_MODE_FIFO_KHR) {
        Some(m) => *m,
        None => vk::PRESENT_MODE_IMMEDIATE_KHR,
      },
    }
  }

  fn init(inst: vk::Instance, pdevice: vk::PhysicalDevice, device: vk::Device, surface: vk::SurfaceKHR) -> Self {
    // surface capabilities
    let mut capabilities = unsafe { std::mem::uninitialized() };
    let ext = vk::InstanceExtensions::new(inst);
    ext.GetPhysicalDeviceSurfaceCapabilitiesKHR(pdevice, surface, &mut capabilities);

    let (format, colorspace) = Self::get_default_format(&ext, pdevice, surface);
    let presentmode = Self::get_default_presentmode(&ext, pdevice, surface);

    Builder {
      device,
      capabilities,
      info: vk::SwapchainCreateInfoKHR {
        sType: vk::STRUCTURE_TYPE_SWAPCHAIN_CREATE_INFO_KHR,
        pNext: std::ptr::null(),
        flags: 0,
        surface,
        minImageCount: match capabilities.maxImageCount {
          0 => capabilities.minImageCount + 1,
          n => n,
        },
        imageFormat: format,
        imageColorSpace: colorspace,
        imageExtent: capabilities.currentExtent,
        imageArrayLayers: 1,
        imageUsage: vk::IMAGE_USAGE_TRANSFER_DST_BIT | vk::IMAGE_USAGE_COLOR_ATTACHMENT_BIT,
        imageSharingMode: vk::SHARING_MODE_EXCLUSIVE,
        queueFamilyIndexCount: 0,
        pQueueFamilyIndices: std::ptr::null(),
        preTransform: capabilities.currentTransform,
        compositeAlpha: vk::COMPOSITE_ALPHA_OPAQUE_BIT_KHR,
        presentMode: presentmode,
        clipped: vk::TRUE,
        oldSwapchain: vk::NULL_HANDLE,
      },
    }
  }

  pub fn colorformat(&mut self, format: vk::Format, colorspace: vk::ColorSpaceKHR) -> &mut Self {
    self.info.imageFormat = format;
    self.info.imageColorSpace = colorspace;
    self
  }
  pub fn presentmode(&mut self, mode: vk::PresentModeKHR) -> &mut Self {
    self.info.presentMode = mode;
    self
  }
  pub fn imageusage(&mut self, usage: vk::ImageUsageFlags) -> &mut Self {
    self.info.imageUsage = usage;
    self
  }
  pub fn extent(&mut self, extent: vk::Extent2D) -> &mut Self {
    self.info.imageExtent = vk::Extent2D {
      width: u32::min(
        self.capabilities.minImageExtent.width,
        u32::max(extent.width, self.capabilities.maxImageExtent.width),
      ),
      height: u32::min(
        self.capabilities.minImageExtent.height,
        u32::max(extent.height, self.capabilities.maxImageExtent.height),
      ),
    };
    self
  }

  pub fn new(self) -> Swapchain {
    let device = self.device;
    let ext = vk::DeviceExtensions::new(device);

    let mut handle = vk::NULL_HANDLE;
    ext.CreateSwapchainKHR(device, &self.info, std::ptr::null(), &mut handle);

    // create image views for the swap chain
    let mut image_count = 0;
    ext.GetSwapchainImagesKHR(device, handle, &mut image_count, std::ptr::null_mut());
    let mut images = Vec::with_capacity(image_count as usize);
    ext.GetSwapchainImagesKHR(device, handle, &mut image_count, images.as_mut_ptr());
    unsafe {
      images.set_len(image_count as usize);
    }

    let mut views = Vec::with_capacity(image_count as usize);
    for i in 0..image_count as usize {
      let info = vk::ImageViewCreateInfo {
        sType: vk::STRUCTURE_TYPE_IMAGE_VIEW_CREATE_INFO,
        pNext: std::ptr::null(),
        flags: 0,
        image: images[i],
        viewType: vk::IMAGE_VIEW_TYPE_2D,
        format: self.info.imageFormat,
        components: vk::ComponentMapping {
          r: vk::COMPONENT_SWIZZLE_IDENTITY,
          g: vk::COMPONENT_SWIZZLE_IDENTITY,
          b: vk::COMPONENT_SWIZZLE_IDENTITY,
          a: vk::COMPONENT_SWIZZLE_IDENTITY,
        },
        subresourceRange: vk::ImageSubresourceRange {
          aspectMask: vk::IMAGE_ASPECT_COLOR_BIT,
          baseMipLevel: 0,
          levelCount: 1,
          baseArrayLayer: 0,
          layerCount: 1,
        },
      };

      let mut view = vk::NULL_HANDLE;
      vk::CreateImageView(device, &info, std::ptr::null(), &mut view);
      views.push(view);
    }

    //VkSemaphoreCreateInfo semaphore_info = {};
    //semaphore_info.sType = VK_STRUCTURE_TYPE_SEMAPHORE_CREATE_INFO;
    //VkSemaphore signal_next_image;
    //CREATE_FAIL(vkCreateSemaphore(dh, &semaphore_info, nullptr, &signal_next_image), "semaphor create failed");

    Swapchain {
      device,
      ext,
      handle,
      images,
      views,
    }
  }
}
