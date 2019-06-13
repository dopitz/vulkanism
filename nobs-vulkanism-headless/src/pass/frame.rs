use super::Framebuffer;
use crate::cmd::stream::*;
use crate::cmd::CmdBuffer;
use crate::cmd::CmdPool;
use crate::cmd::RRBatch;
use vk;

pub struct Frame<'a> {
  pub cmds: CmdPool,
  pub batch: &'a mut RRBatch,
  pub cs: CmdBuffer,
}

impl<'a> Frame<'a> {
  pub fn new(cmds: CmdPool, batch: &'a mut RRBatch) -> Self {
    batch.next().unwrap();
    let cs = cmds.begin_stream().unwrap();
    Self { cmds, batch, cs }
  }

  pub fn submit(self, queue: vk::Queue) -> vk::Semaphore {
    let (_, wait) = self.batch.push(self.cs).submit(queue);
    wait.unwrap()
  }
}

impl<'a> Stream for Frame<'a> {
  fn push<T: StreamPush>(mut self, o: &T) -> Self {
    self.cs = self.cs.push(o);
    self
  }
}
impl<'a> StreamMut for Frame<'a> {
  fn push_mut<T: StreamPushMut>(mut self, o: &mut T) -> Self {
    self.cs = self.cs.push_mut(o);
    self
  }
}
