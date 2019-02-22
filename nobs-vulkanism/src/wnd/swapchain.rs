use vk;

#[derive(Debug)]
pub struct NextImage {
  pub signal: vk::Semaphore,
  pub index: u32,
}

pub struct Swapchain {
  device: vk::Device,
  ext: vk::DeviceExtensions,
  sig_index: usize,
  pub extent: vk::Extent2D,
  pub handle: vk::SwapchainKHR,
  pub images: Vec<vk::Image>,
  pub views: Vec<vk::ImageView>,
  pub signals: Vec<vk::Semaphore>,
}

impl Drop for Swapchain {
  fn drop(&mut self) {
    for v in self.views.iter() {
      vk::DestroyImageView(self.device, *v, std::ptr::null());
    }
    for s in self.signals.iter() {
      vk::DestroySemaphore(self.device, *s, std::ptr::null());
    }
    self.ext.DestroySwapchainKHR(self.device, self.handle, std::ptr::null());
  }
}

impl Swapchain {
  pub fn build(inst: vk::Instance, pdevice: vk::PhysicalDevice, device: vk::Device, surface: vk::SurfaceKHR) -> Builder {
    Builder::new(inst, pdevice, device, surface)
  }

  pub fn next_image(&mut self) -> NextImage {
    let signal = self.signals[self.sig_index];
    let mut index = 0;
    self.sig_index = (self.sig_index + 1) % self.signals.len();
    self
      .ext
      .AcquireNextImageKHR(self.device, self.handle, u64::max_value(), signal, vk::NULL_HANDLE, &mut index);
    NextImage { signal, index }
  }

  pub fn blit(&self, index: u32, src: vk::Image) -> vk::cmd::commands::Blit {
    vk::cmd::commands::Blit::new()
      .src(src)
      .src_offset_end(self.extent.width as i32, self.extent.height as i32, 1)
      .dst(self.images[index as usize])
      .dst_offset_end(self.extent.width as i32, self.extent.height as i32, 1)
  }

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

  fn new(inst: vk::Instance, pdevice: vk::PhysicalDevice, device: vk::Device, surface: vk::SurfaceKHR) -> Self {
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

  pub fn create(self) -> Swapchain {
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

    let mut signals = Vec::with_capacity(image_count as usize);
    for _i in 0..image_count as usize {
      let mut sig = vk::NULL_HANDLE;
      let info = vk::SemaphoreCreateInfo {
        sType: vk::STRUCTURE_TYPE_SEMAPHORE_CREATE_INFO,
        pNext: std::ptr::null(),
        flags: 0,
      };
      vk::CreateSemaphore(device, &info, std::ptr::null(), &mut sig);
      signals.push(sig);
    }

    Swapchain {
      device,
      ext,
      sig_index: 0,
      extent: self.info.imageExtent,
      handle,
      images,
      views,
      signals,
    }
  }
}
