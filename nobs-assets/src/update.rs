use vk::cmd::commands::BufferBarrier;
use vk::cmd::commands::BufferCopy;
use vk::cmd::commands::BufferImageCopy;
use vk::cmd::commands::ImageBarrier;
use vk::cmd::stream::*;
use vk::mem::StagingFrame;
use vk::mem::StagingRange;

/// Updates buffers and images
///
/// We can easily enqueue buffer and image updates by
///  1. [getting a cached staging buffer](struct.Update.html#method.get_staging) from `Update`.
///  2. transfering our data into the staging buffer
///  3. [pushing the copy operation](struct.Update.html#push_buffer)
///
/// `Update` implements pushing into command buffers. This will run all buffer copies and memory barriers.
/// The staging buffers are managed internally and will be cached and cleand up automatically
pub struct Update {
  buffer_copy: Vec<(BufferCopy, Option<BufferBarrier>)>,
  texture_copy: Vec<(BufferImageCopy, Option<ImageBarrier>)>,

  staging: StagingFrame,
  next_staging: Option<StagingFrame>,

  cmdpool: vk::cmd::CmdPool,
  queue: vk::device::Queue,
  batch: vk::cmd::AutoBatch,
}

impl Update {
  pub fn new(device: vk::Device, queue: vk::device::Queue, mem: vk::mem::Mem) -> Self {
    Self {
      buffer_copy: Default::default(),
      texture_copy: Default::default(),

      staging: StagingFrame::new(mem, 1 << 23).unwrap(),
      next_staging: None,

      cmdpool: vk::cmd::CmdPool::new(device, queue.family).unwrap(),
      queue,
      batch: vk::cmd::AutoBatch::new(device).unwrap(),
    }
  }

  pub fn get_staging(&mut self, size: vk::DeviceSize) -> StagingRange {
    match self.staging.next(size) {
      Ok(range) => range,
      Err(_) => {
        let mut s = StagingFrame::new(self.staging.get_mem(), self.staging.capacity() + size).expect("Out of memory for staging buffer");
        let range = s.next(size).unwrap();
        self.next_staging = Some(s);
        range
      }
    }
  }
  pub fn push_buffer(&mut self, c: BufferCopy, b: Option<BufferBarrier>) -> &mut Self {
    self.buffer_copy.push((c, b));
    self
  }
  pub fn push_image(&mut self, c: BufferImageCopy, b: Option<ImageBarrier>) -> &mut Self {
    self.texture_copy.push((c, b));
    self
  }

  pub fn blocking_buffer_update<T>(&mut self, src: &[T], dst: vk::Buffer) {
    self.staging.push_state();
    let mut stage = self.get_staging(vk::DeviceSize::max(self.staging.remaining_size(), 1 << 20));

    let n = (stage.size() / vk::mem::device_size!(T)) as usize;
    let size = vk::mem::device_size!(n, T);

    let mut start = 0;
    let mut offset = 0;
    while start < src.len() {
      let src = &src[start..usize::min(start + n, src.len())];
      start += n;

      let map = stage.map().unwrap();
      map.host_to_device_slice(src);
      let cpy = stage.copy_into_buffer(dst, offset);
      offset += size;

      self
        .batch
        .push(self.cmdpool.begin_stream().unwrap().push(&cpy))
        .submit_immediate(self.queue.handle)
        .unwrap();
    }

    self.staging.pop_state();
  }

  pub fn get_mem(&self) -> vk::mem::Mem {
    self.staging.get_mem()
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

    if let Some(s) = self.next_staging.take() {
      self.staging = s;
    }
    self.staging.reset();
    cs
  }
}
