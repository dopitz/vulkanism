use super::Pass;
use super::PassId;
use crate::fb::Framebuffer;
use crate::wnd::swapchain::NextImage;
use crate::wnd::Swapchain;
use vk;

pub struct Frame<'a> {
  cmds: vk::cmd::Pool,
  next: NextImage,
  sc: &'a mut Swapchain,
  fb: &'a mut Framebuffer,
  frame: &'a mut vk::cmd::Frame,
}

impl<'a> Frame<'a> {
  pub fn new(cmds: vk::cmd::Pool, next: NextImage, sc: &'a mut Swapchain, fb: &'a mut Framebuffer, frame: &'a mut vk::cmd::Frame) -> Self {
    Self { cmds, frame, sc, fb, next }
  }

  pub fn push<P: PassId, T: Pass<P>>(mut self, mut pass: T) -> Self {
    pass.run(self.cmds.clone(), &mut self.frame);
    self
  }

  pub fn present(self) -> Self {
    //let (_, wait) = self
    //  .frame
    //  .wait_for(self.next.signal, vk::PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT)
    //  .submit(self.renderer.device.queues[0].handle);

    //sc.present(self.renderer.device.queues[0].handle, self.next.index, &[wait.unwrap()]);
    self
  }
}
