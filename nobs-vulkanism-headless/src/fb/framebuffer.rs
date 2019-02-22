use vk;

use crate::cmd::commands::RenderpassBegin;
use crate::cmd::commands::RenderpassEnd;
use crate::fb::Renderpass;
use crate::mem;

pub struct Framebuffer {
  device: vk::Device,
  pub pass: vk::RenderPass,
  pub handle: vk::Framebuffer,

  pub extent: vk::Extent2D,
  pub images: Vec<vk::Image>,
  pub views: Vec<vk::ImageView>,
  pub clear: Vec<vk::ClearValue>,
}

impl Framebuffer {
  pub fn set_clear(&mut self, clear: &[vk::ClearValue]) {
    assert!(self.clear.len() == clear.len());
    for (c, s) in self.clear.iter_mut().zip(clear.iter()) {
      *c = *s;
    }
  }

  pub fn begin_area(&self, area: vk::Rect2D) -> RenderpassBegin {
    RenderpassBegin::new(self.pass, self.handle).clear(&self.clear).area(area)
  }
  pub fn begin(&self) -> RenderpassBegin {
    RenderpassBegin::new(self.pass, self.handle).clear(&self.clear).extent(self.extent)
  }
  pub fn end(&self) -> RenderpassEnd {
    RenderpassEnd {}
  }
}

impl Drop for Framebuffer {
  fn drop(&mut self) {
    vk::DestroyFramebuffer(self.device, self.handle, std::ptr::null());

    for v in self.views.iter() {
      vk::DestroyImageView(self.device, *v, std::ptr::null())
    }
  }
}

pub struct Builder {
  device: vk::Device,
  pass: vk::RenderPass,
  extent: vk::Extent2D,
  images: Vec<vk::Image>,
  views: Vec<vk::ImageView>,
  clear: Vec<vk::ClearValue>,
}

impl Builder {
  pub fn new(device: vk::Device, pass: vk::RenderPass) -> Self {
    Self {
      device,
      pass,
      extent: vk::Extent2D { width: 0, height: 0 },
      images: Default::default(),
      views: Default::default(),
      clear: Default::default(),
    }
  }

  pub fn extent(&mut self, extent: vk::Extent2D) -> &mut Self {
    self.extent = extent;
    self
  }

  pub fn target(&mut self, image: vk::Image, view: vk::ImageView, clear: vk::ClearValue) -> &mut Self {
    self.images.push(image);
    self.views.push(view);
    self.clear.push(clear);
    self
  }

  pub fn create(&mut self) -> Framebuffer {
    let info = vk::FramebufferCreateInfo {
      sType: vk::STRUCTURE_TYPE_FRAMEBUFFER_CREATE_INFO,
      pNext: std::ptr::null(),
      flags: 0,
      renderPass: self.pass,
      attachmentCount: self.views.len() as u32,
      pAttachments: self.views.as_ptr(),
      width: self.extent.width,
      height: self.extent.height,
      layers: 1,
    };

    let mut handle = vk::NULL_HANDLE;
    vk::CreateFramebuffer(self.device, &info, std::ptr::null(), &mut handle);
    Framebuffer {
      device: self.device,
      pass: self.pass,
      handle,
      extent: self.extent,
      images: self.images.clone(),
      views: self.views.clone(),
      clear: self.clear.clone(),
    }
  }
}

pub struct RenderpassFramebufferBuilder<'a, 'b> {
  alloc: &'a mut mem::Allocator,
  pass: &'b Renderpass,
  images: Vec<vk::Image>,
  extent: vk::Extent2D,
}

impl<'a, 'b> RenderpassFramebufferBuilder<'a, 'b> {
  pub fn new(pass: &'b Renderpass, alloc: &'a mut mem::Allocator) -> Self {
    let mut images = Vec::new();
    images.resize(pass.attachments.len(), vk::NULL_HANDLE);
    Self {
      alloc,
      pass,
      images,
      extent: vk::Extent2D { width: 0, height: 0 },
    }
  }

  pub fn target(mut self, index: usize, image: vk::Image) -> Self {
    debug_assert!(index < self.images.len());
    self.images[index] = image;
    self
  }

  pub fn extent(mut self, extent: vk::Extent2D) -> Self {
    self.extent = extent;
    self
  }

  pub fn create(mut self) -> Framebuffer {
    fn is_depth(format: vk::Format) -> bool {
      crate::fb::DEPTH_FORMATS.iter().find(|f| **f == format).is_some()
    }

    // create images for every one that was not set externally
    let mut builder = mem::Resource::new();
    for (i, f) in self
      .images
      .iter_mut()
      .zip(self.pass.attachments.iter())
      .filter_map(|(i, a)| match *i {
        vk::NULL_HANDLE => Some((i, a.format)),
        _ => None,
      })
    {
      builder = match is_depth(f) {
        true => builder.new_image(i).depth_attachment(self.extent.width, self.extent.height, f),
        false => builder.new_image(i).color_attachment(self.extent.width, self.extent.height, f),
      }
      .submit();
    }
    builder.bind(self.alloc, mem::BindType::Block).unwrap();

    // create view for every image
    let mut views = Vec::with_capacity(self.images.len());
    for (i, f) in self.images.iter().zip(self.pass.attachments.iter()).map(|(i, a)| (i, a.format)) {
      let builder = mem::ImageView::new(self.pass.device, *i).format(f);
      let view = match is_depth(f) {
        true => builder.aspect(vk::IMAGE_ASPECT_DEPTH_BIT),
        false => builder.aspect(vk::IMAGE_ASPECT_COLOR_BIT),
      }
      .create()
      .unwrap();
      views.push(view);
    }

    // create the framebuffer
    let mut builder = crate::fb::new_framebuffer(self.pass.device, self.pass.pass);
    for (i, v, f) in self
      .images
      .iter()
      .zip(views.iter())
      .zip(self.pass.attachments.iter())
      .map(|((i, v), a)| (i, v, a.format))
    {
      builder.target(
        *i,
        *v,
        match is_depth(f) {
          true => crate::fb::clear_depth(0.0),
          false => crate::fb::clear_coloru32([0, 0, 0, 0]),
        },
      );
    }
    builder.extent(self.extent).create()
  }
}
