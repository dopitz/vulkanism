use vk::cmd::commands::BufferBarrier;
use vk::cmd::commands::BufferCopy;
use vk::cmd::commands::BufferImageCopy;
use vk::cmd::commands::ImageBarrier;
use vk::cmd::stream::*;
use vk::mem::StagingFrame;
use vk::mem::StagingRange;

/// Updates buffers and images
///
/// This class uses staging memory to transfer data to and from the device.
/// The stanging memory starts by default with 8Mb and is dynamically grown in case it is too small.
///
/// We can use the cached staging memory in two ways
///  1. Blocking updates (
///     [blocking_buffer_update](struct.Update.html#method.blocking_buffer_update),
///     [blocking_buffer_download](struct.Update.html#method.blocking_buffer_download))
///     will use as much staging memory as currently available in the cache and transfer data batch-wise.
///     These functions will return after the transfer job has been completed
///  2. Non-blocking updates (
///     [buffer_update](struct.Update.html#method.buffer_update))
///     will allocate enough staging memory to hold the complete data to be transfered and delay the copy from staging memory to the respective buffer/image.
///     The buffer/image will be updated when [Update](struct.Update.html) is enqueued into a command buffer.
///     We can use [push_buffer](struct.Update.html#method.push_buffer) and [push_image](struct.Update.html#method.push_image) to manually
///     inject buffer/image copy commands to be executed when `Update` is enqueued into a command buffer.
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
  /// Create a new `Update` manager
  ///
  /// See [with_size](struct.Update.html#method.with_size) for details.
  /// Uses 8Mb as size for the staging memory
  pub fn new(device: vk::Device, queue: vk::device::Queue, mem: vk::mem::Mem) -> Self {
    Self::with_size(device, queue, mem, 1 << 23)
  }

  /// Create a new `Update` manager
  ///
  /// # Arguments
  /// * `device` - device handle
  /// * `queue` - queue used to submit blocking updates
  /// * `mem` - memory manager to allocate the staging memory
  /// * `size` - starting size of the staging memory
  pub fn with_size(device: vk::Device, queue: vk::device::Queue, mem: vk::mem::Mem, size: vk::DeviceSize) -> Self {
    Self {
      buffer_copy: Default::default(),
      texture_copy: Default::default(),

      staging: StagingFrame::new(mem, size).unwrap(),
      next_staging: None,

      cmdpool: vk::cmd::CmdPool::new(device, queue.family).unwrap(),
      queue,
      batch: vk::cmd::AutoBatch::new(device).unwrap(),
    }
  }

  /// Get a piece of staging memory
  ///
  /// Returns a piece of staging memory at least of the requested size.
  /// Grows the internal staging buffer if there is not enough staging memory left.
  ///
  /// Makes the staging memory available again when `Update` is enqueued into a command buffer.
  ///
  /// # Arguments
  ///  * `size` - size of the requested staging buffer in bytes
  ///
  /// # Returns
  /// A range into the cached staging memory
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
  /// Push buffer copy command to be executed the next time `Update` is enqueued into a command buffer
  ///
  /// # Arguments
  ///  * `c` - BufferCopy command
  ///  * `b` - Optional barrier for this copy operation
  pub fn push_buffer(&mut self, c: BufferCopy, b: Option<BufferBarrier>) -> &mut Self {
    self.buffer_copy.push((c, b));
    self
  }
  /// Push image copy command to be executed the next time `Update` is enqueued into a command buffer
  ///
  /// # Arguments
  ///  * `c` - ImageCopy command
  ///  * `b` - Optional barrier for this copy operation
  pub fn push_image(&mut self, c: BufferImageCopy, b: Option<ImageBarrier>) -> &mut Self {
    self.texture_copy.push((c, b));
    self
  }

  /// Update buffer
  ///
  /// Copies `src` into the staging memory.
  /// Copies the staging memory into `dst` the next time `Update` is enqueued into a command buffer.
  ///
  /// # Arguments
  ///  * `src` - slice from which data is read
  ///  * `dst` - vulkan Handle of the buffer to be updated
  ///  * `barrier` - Optional barrier for this copy operation
  pub fn buffer_update<T>(&mut self, src: &[T], dst: vk::Buffer, barrier: Option<BufferBarrier>) {
    let mut stage = self.get_staging(vk::mem::device_size!(src.len(), T));
    let map = stage.map().unwrap();
    map.host_to_device_slice(&src);

    self.push_buffer(stage.copy_into_buffer(dst, 0), barrier);
  }

  /// Update buffer synchronously
  ///
  /// The `src` is copied via the largest remaining staging memory into `dst`.
  /// Performs multiple updates batch-wise.
  ///
  /// # Arguments
  ///  * `src` - slice from which data is read
  ///  * `dst` - vulkan Handle of the buffer to be updated
  pub fn blocking_buffer_update<T>(&mut self, src: &[T], dst: vk::Buffer) {
    self.staging.push_state();
    let mut stage = self.get_staging(vk::DeviceSize::max(self.staging.remaining_size(), 1 << 20));

    let n = (stage.size() / vk::mem::device_size!(T)) as usize;
    let size = vk::mem::device_size!(n, T);

    let mut start = 0;
    let mut offset = 0;
    let slen = src.len();
    while start < src.len() {
      let src = &src[start..usize::min(start + n, src.len())];
      start += n;

      let map = stage.map().unwrap();
      map.host_to_device_slice(src);
      let cpy = stage
        .range(0, vk::DeviceSize::min(size, vk::mem::device_size!(slen, T) - offset))
        .copy_into_buffer(dst, offset);
      offset += size;

      self
        .batch
        .push(self.cmdpool.begin_stream().unwrap().push(&cpy))
        .submit_immediate(self.queue.handle)
        .unwrap();
    }

    self.staging.pop_state();
  }
  /// Update buffer synchronously
  ///
  /// The `src` is copied via the largest remaining staging memory into `dst`.
  /// Performs multiple updates batch-wise.
  ///
  /// # Arguments
  ///  * `src` - vulkan Handle of the buffer to be downloaded
  ///  * `dst` - slice to which data is written
  pub fn blocking_buffer_download<T>(&mut self, src: vk::Buffer, dst: &mut [T]) {
    self.staging.push_state();
    let mut stage = self.get_staging(vk::DeviceSize::max(self.staging.remaining_size(), 1 << 20));

    let n = (stage.size() / vk::mem::device_size!(T)) as usize;
    let size = vk::mem::device_size!(n, T);

    let mut start = 0;
    let mut offset = 0;
    while start < dst.len() {
      let cpy = stage.copy_from_buffer(src, offset);
      offset += size;

      self
        .batch
        .push(self.cmdpool.begin_stream().unwrap().push(&cpy))
        .submit_immediate(self.queue.handle)
        .unwrap();

      let len = dst.len();
      let dst = &mut dst[start..usize::min(start + n, len)];
      start += n;
      let map = stage.map().unwrap();
      map.device_to_host_slice(dst);
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
