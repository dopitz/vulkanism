use vk::builder::Buildable;
use vk::cmd::stream::*;
use vk::mem::Handle;
use vk::pass::DrawPass;
use vk::pass::Framebuffer;
use vk::pass::Renderpass;

pub struct Select {
  rp: Renderpass,
  fb: Framebuffer,
  pass: DrawPass,
  mem: vk::mem::Mem,
}

impl Select {
  pub fn new(device: vk::Device, extent: vk::Extent2D, mem: vk::mem::Mem) -> Self {
    let mut mem = mem.clone();
    let rp = vk::pass::Renderpass::build(device)
      .attachment(
        0,
        vk::AttachmentDescription::build()
          .format(vk::FORMAT_R32_UINT)
          .initial_layout(vk::IMAGE_LAYOUT_COLOR_ATTACHMENT_OPTIMAL),
      )
      .subpass(
        0,
        vk::SubpassDescription::build().bindpoint(vk::PIPELINE_BIND_POINT_GRAPHICS).color(0),
      )
      .dependency(vk::SubpassDependency::build().external(0))
      .create()
      .unwrap();

    let fb = vk::pass::Framebuffer::build_from_pass(&rp, &mut mem.alloc).extent(extent).create();

    Self {
      rp,
      fb,
      pass: DrawPass::new(),
      mem,
    }
  }

  pub fn resize(&mut self, size: vk::Extent2D) {
    self.mem.alloc.destroy(Handle::Image(self.fb.images[0]));
    self.fb = vk::pass::Framebuffer::build_from_pass(&self.rp, &mut self.mem.alloc)
      .extent(size)
      .create();
  }
}

impl StreamPushMut for Select {
  fn enqueue_mut(&mut self, mut cs: CmdBuffer) -> CmdBuffer {
    cs.push(&self.pass)
  }
}
