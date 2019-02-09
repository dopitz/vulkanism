use crate::cmd;
use vk;

pub struct Framebuffer {
  device: vk::Device,
  pass: vk::RenderPass,
  fb: vk::Framebuffer,

  extent: vk::Extent2D,
  images: Vec<vk::Image>,
  views: Vec<vk::ImageView>,
  clear: Vec<vk::ClearValue>,
}

impl Framebuffer {
  pub fn set_clear(&mut self, clear: &[vk::ClearValue]) {
    assert!(self.clear.len() == clear.len());
    for (c, s) in self.clear.iter_mut().zip(clear.iter()) {
      *c = *s;
    }
  }

  pub fn begin_area(&self, area: vk::Rect2D) -> cmd::RenderpassBegin {
    *cmd::RenderpassBegin::new(self.pass, self.fb).clear(&self.clear).area(area)
  }
  pub fn begin(&self) -> cmd::RenderpassBegin {
    *cmd::RenderpassBegin::new(self.pass, self.fb).clear(&self.clear).extent(self.extent)
  }
  pub fn end(&self) -> cmd::RenderpassEnd {
    cmd::RenderpassEnd {}
  }

  pub fn get_pass(&self) -> vk::RenderPass {
    self.pass
  }
  pub fn get_fb(&self) -> vk::Framebuffer {
    self.fb
  }

  pub fn get_images(&self) -> &[vk::Image] {
    &self.images
  }
  pub fn get_views(&self) -> &[vk::ImageView] {
    &self.views
  }
}

impl Drop for Framebuffer {
  fn drop(&mut self) {
    vk::DestroyRenderPass(self.device, self.pass, std::ptr::null());
    vk::DestroyFramebuffer(self.device, self.fb, std::ptr::null());

    for v in self.views.iter() {
      vk::DestroyImageView(self.device, *v, std::ptr::null())
    }
  }
}

pub struct Builder {
  fb: Framebuffer,
}

impl Builder {
  pub fn new(device: vk::Device, pass: vk::RenderPass) -> Self {
    Self {
      fb: Framebuffer {
        device,
        pass,
        fb: vk::NULL_HANDLE,
        extent: vk::Extent2D { width: 0, height: 0 },
        images: Default::default(),
        views: Default::default(),
        clear: Default::default(),
      },
    }
  }

  pub fn extent(&mut self, extent: vk::Extent2D) -> &mut Self {
    self.fb.extent = extent;
    self
  }

  pub fn target(&mut self, image: vk::Image, view: vk::ImageView, clear: vk::ClearValue) -> &mut Self {
    self.fb.images.push(image);
    self.fb.views.push(view);
    self.fb.clear.push(clear);
    self
  }

  pub fn create(mut self) -> Framebuffer {
    let info = vk::FramebufferCreateInfo {
      sType: vk::STRUCTURE_TYPE_FRAMEBUFFER_CREATE_INFO,
      pNext: std::ptr::null(),
      flags: 0,
      renderPass: self.fb.pass,
      attachmentCount: self.fb.views.len() as u32,
      pAttachments: self.fb.views.as_ptr(),
      width: self.fb.extent.width,
      height: self.fb.extent.height,
      layers: 1,
    };

    vk::CreateFramebuffer(self.fb.device, &info, std::ptr::null(), &mut self.fb.fb);
    self.fb
  }
}
