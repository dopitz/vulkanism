use vk::cmd::commands::BufferBarrier;
use vk::cmd::commands::BufferCopy;
use vk::cmd::commands::BufferImageCopy;
use vk::cmd::commands::ImageBarrier;
use vk::cmd::stream::*;
use vk::mem::Staging;
use vk::mem::StagingRange;

pub struct Update {
  // TODO: For some reason StagingFrame does not work...
  // It is kind of inherently bad to create lots of small buffers and delete them as soon as we done copying
  // However we do net do this too much... yet
  pub mem: vk::mem::Mem,
  stagings: Vec<Staging>,
  buffer_copy: Vec<(BufferCopy, Option<BufferBarrier>)>,
  texture_copy: Vec<(BufferImageCopy, Option<ImageBarrier>)>,
}

impl Update {
  pub fn new(mem: vk::mem::Mem) -> Self {
    Self {
      mem,
      stagings: Default::default(),
      buffer_copy: Default::default(),
      texture_copy: Default::default(),
    }
  }
  pub fn get_staging(&mut self, size: vk::DeviceSize) -> StagingRange {
    let s = Staging::new(self.mem.clone(), size).unwrap();
    let range = s.range(0, size);
    self.stagings.push(s);
    range
  }
  pub fn push_buffer(&mut self, c: BufferCopy, b: Option<BufferBarrier>) -> &mut Self {
    self.buffer_copy.push((c, b));
    self
  }
  pub fn push_image(&mut self, c: BufferImageCopy, b: Option<ImageBarrier>) -> &mut Self {
    self.texture_copy.push((c, b));
    self
  }
}

impl StreamPushMut for Update {
  fn enqueue_mut(&mut self, mut cs: CmdBuffer) -> CmdBuffer {
    for (copy, barrier) in self.buffer_copy.iter() {
      cs = cs.push(copy).push_if(barrier);
    }
    for (copy, barrier) in self.texture_copy.iter() {
      cs = cs.push(copy).push_if(barrier);
    }
    self.buffer_copy.clear();
    self.texture_copy.clear();
    self.stagings.clear();
    cs
  }
}
