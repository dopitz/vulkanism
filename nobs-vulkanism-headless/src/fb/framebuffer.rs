use vk;

use crate::cmd::commands::RenderpassBegin;
use crate::cmd::commands::RenderpassEnd;
use crate::fb::Renderpass;
use crate::mem;
use crate::util;
use vk::builder::Buildable;

/// Wrapper for a vulkan framebuffer
///
/// Owns the images and image views for all framebuffer attachments
pub struct Framebuffer {
  device: vk::Device,
  pub pass: vk::RenderPass,
  pub handle: vk::Framebuffer,

  pub extent: vk::Extent2D,
  pub images: Vec<vk::Image>,
  pub views: Vec<vk::ImageView>,
  pub clear: Vec<vk::ClearValue>,
}

impl Drop for Framebuffer {
  fn drop(&mut self) {
    vk::DestroyFramebuffer(self.device, self.handle, std::ptr::null());

    for v in self.views.iter() {
      vk::DestroyImageView(self.device, *v, std::ptr::null())
    }
  }
}

impl Framebuffer {
  pub fn build(device: vk::Device, pass: vk::RenderPass) -> Builder {
    Builder::new(device, pass)
  }

  pub fn build_from_pass<'a, 'b>(pass: &'b Renderpass, alloc: &'a mut mem::Allocator) -> RenderpassFramebufferBuilder<'a, 'b> {
    RenderpassFramebufferBuilder::new(pass, alloc)
  }

  /// Set the clear values for all attachments
  pub fn set_clear(&mut self, clear: &[vk::ClearValue]) {
    assert!(self.clear.len() == clear.len());
    for (c, s) in self.clear.iter_mut().zip(clear.iter()) {
      *c = *s;
    }
  }

  /// Returns a render pass begin command
  ///
  /// The returned command is configured to draw in the specified `area`.
  pub fn begin_area(&self, area: vk::Rect2D) -> RenderpassBegin {
    RenderpassBegin::new(self.pass, self.handle).clear(&self.clear).area(area)
  }
  /// Returns a render pass begin command
  pub fn begin(&self) -> RenderpassBegin {
    RenderpassBegin::new(self.pass, self.handle).clear(&self.clear).extent(self.extent)
  }
  /// Returns a render pass end command
  pub fn end(&self) -> RenderpassEnd {
    RenderpassEnd {}
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

/// Builder for [Framebuffer](struct.Framebuffer.html)
impl Builder {
  /// Build a new framebuffer for the specified renderpass
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

  /// Sets the extend of all framebuffer attachments
  pub fn extent(&mut self, extent: vk::Extent2D) -> &mut Self {
    self.extent = extent;
    self
  }

  /// Adds the `image` as framebuffer attachment
  ///
  /// Attachments are added in sequence, meaning the attachment at position 0 is set by the first call to `target`, the one at position 1 by the second call, etc.
  pub fn target(&mut self, image: vk::Image, view: vk::ImageView, clear: vk::ClearValue) -> &mut Self {
    self.images.push(image);
    self.views.push(view);
    self.clear.push(clear);
    self
  }

  /// Creates the framebuffer
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

/// Builder for [Framebuffer](struct.Framebuffer.html)
///
/// This builder is specialized in a way that we do not need to specify any images as attachments.
/// For attachments in the renderpass that have not been externally specified with [target](struct.RenderpassFramebufferBuilder.html#method.target)
/// the builder creates new images automatically.
pub struct RenderpassFramebufferBuilder<'a, 'b> {
  alloc: &'a mut mem::Allocator,
  pass: &'b Renderpass,
  images: Vec<vk::Image>,
  extent: vk::Extent2D,
}

impl<'a, 'b> RenderpassFramebufferBuilder<'a, 'b> {
  /// Build a new framebuffer for the specified renderpass
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

  /// Specify `image` as rendertarget at position `index`.
  ///
  /// The builder will use the specified image as attachment.
  /// For the attachment at position `index` no image will be created in [create](struct.RenderpassFramebufferBuilder.html#method.create).
  /// An image view will still be created.
  pub fn target(mut self, index: usize, image: vk::Image) -> Self {
    debug_assert!(index < self.images.len());
    self.images[index] = image;
    self
  }

  /// Specyfy the extent of all attachments
  pub fn extent(mut self, extent: vk::Extent2D) -> Self {
    self.extent = extent;
    self
  }

  /// Creates the Framebuffer
  ///
  /// This will create an image and image view for every attachment, except an image has been [set](struct.RenderpassFramebufferBuilder.html#method.target).
  /// Images are created with the [Allocator](../../mem/struct.Allocator.html) that has been specified in the builders constructor.
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
      let builder = vk::ImageViewCreateInfo::build().image(*i).format(f);
      let view = match is_depth(f) {
        true => builder.aspect(vk::IMAGE_ASPECT_DEPTH_BIT),
        false => builder.aspect(vk::IMAGE_ASPECT_COLOR_BIT),
      }
      .create(self.pass.device)
      .unwrap();
      views.push(view);
    }

    // create the framebuffer
    let mut builder = Framebuffer::build(self.pass.device, self.pass.pass);
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
          true => crate::fb::clear_depth(1.0),
          false => crate::fb::clear_coloru32([0, 0, 0, 0]),
        },
      );
    }
    builder.extent(self.extent).create()
  }
}
