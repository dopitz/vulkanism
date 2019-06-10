use super::Framebuffer;
use crate::cmd::commands::StreamPush;
use crate::cmd::Frame as Batch;
use crate::cmd::Pool as CmdPool;
use crate::cmd::Stream;
use vk;

pub struct Frame<'a> {
  pub cmds: CmdPool,
  pub batch: &'a mut Batch,
  pub cs: Stream,
}

impl<'a> Frame<'a> {
  pub fn new(cmds: CmdPool, batch: &'a mut Batch) -> Self {
    batch.next().unwrap();
    let cs = cmds.begin_stream().unwrap();
    Self { cmds, batch, cs }
  }

  pub fn push<T: StreamPush>(mut self, cmd: &T) -> Self {
    self.cs = self.cs.push(cmd);
    self
  }

  pub fn submit(self, queue: vk::Queue) -> vk::Semaphore {
    let (_, wait) = self.batch.push(self.cs).submit(queue);
    wait.unwrap()
  }
}
