use crate::wnd::swapchain::NextImage;
use crate::wnd::Swapchain;
use vk::cmd::stream::*;
use vk::cmd::CmdPool;
use vk::cmd::RRBatch;
use vk::pass::Frame;

pub struct PresentFrame<'a> {
  frame: Frame<'a>,
  next: NextImage,
  sc: &'a mut Swapchain,
}

impl<'a> PresentFrame<'a> {
  pub fn new(cmds: CmdPool, sc: &'a mut Swapchain, batch: &'a mut RRBatch) -> Self {
    let next = sc.next_image();
    let frame = Frame::new(cmds, batch);
    Self { frame, next, sc }
  }

  pub fn present(mut self, queue: vk::Queue, image: vk::Image) {
    self.frame = self.frame.push(&self.sc.blit(self.next.index, image));
    self
      .frame
      .batch
      .wait_for(self.next.signal, vk::PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT);
    let wait = self.frame.submit(queue);
    self.sc.present(queue, self.next.index, &[wait]);
  }
}

impl<'a> Stream for PresentFrame<'a> {
  fn push<T: StreamPush>(mut self, o: &T) -> Self {
    self.frame = self.frame.push(o);
    self
  }
}
impl<'a> StreamMut for PresentFrame<'a> {
  fn push_mut<T: StreamPushMut>(mut self, o: &mut T) -> Self {
    self.frame = self.frame.push_mut(o);
    self
  }
}
