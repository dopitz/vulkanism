use vk;

use crate::cmd::Error;
use crate::cmd::Stream;

struct Batch {
  device: vk::Device,
  streams: Vec<Stream>,
  buffers: Vec<vk::CommandBuffer>,

  wait_signals: Vec<vk::Semaphore>,
  wait_stages: Vec<vk::ShaderStageFlags>,
  signals: vk::Semaphore,
  fence: vk::Fence,
}

/// Collection of streams that are submitted to the same queue.
///
/// Queue submit calles should subit as many command buffers as possible batched together, because it is a very overhead heavy operation.
/// BatchSubmit collects [Streams](struct.Stream.html) that can be batched together and [submits](struct.BatchSubmit.html#method.submit) them in a single call.
///
/// Submitting the batch will yield a [BatchWait](struct.BatchWait.html) to synchronize with the CPU.
/// Syncing with the CPU will yield a BatchSubmit again. This is implemented in a way that the internal vectors holding the streams does not need to be freed/allocated again.
///
/// See [AutoBatch](struct.AutoBatch.html), where this ping-pong behavior is implemented in a single type.
pub struct BatchSubmit {
  batch: Batch,
}

/// Collection of streams that are beeing executed.
///
/// After [Streams](struct.Stream.html) have been submitted to a queue with [BatchSubmit](struct.BatchSubmit.html) they are executed on the device.
/// The single purpose of BatchWait is to [sync](struct.BatchWait.html#method.sync) the execution with the CPU.
pub struct BatchWait {
  batch: Batch,
}

impl Drop for Batch {
  fn drop(&mut self) {
    self.sync().unwrap();
    vk::DestroyFence(self.device, self.fence, std::ptr::null());
    vk::DestroySemaphore(self.device, self.signals, std::ptr::null());
  }
}

impl Batch {
  pub fn new(device: vk::Device) -> Result<Self, Error> {
    Self::with_capacity(device, 1)
  }

  pub fn with_capacity(device: vk::Device, capacity: usize) -> Result<Self, Error> {
    let signals = {
      let info = vk::SemaphoreCreateInfo {
        sType: vk::STRUCTURE_TYPE_SEMAPHORE_CREATE_INFO,
        pNext: std::ptr::null(),
        flags: 0,
      };
      let mut h = vk::NULL_HANDLE;
      vk_check!(vk::CreateSemaphore(device, &info, std::ptr::null(), &mut h)).map_err(|e| Error::CreateStreamFailed(e))?;
      h
    };

    let fence = {
      let info = vk::FenceCreateInfo {
        sType: vk::STRUCTURE_TYPE_FENCE_CREATE_INFO,
        pNext: std::ptr::null(),
        flags: vk::FENCE_CREATE_SIGNALED_BIT,
      };
      let mut h = vk::NULL_HANDLE;
      vk_check!(vk::CreateFence(device, &info, std::ptr::null(), &mut h)).map_err(|e| Error::CreateStreamFailed(e))?;
      h
    };

    Ok(Self {
      device,
      streams: Vec::with_capacity(capacity),
      buffers: Vec::with_capacity(capacity),
      wait_signals: Default::default(),
      wait_stages: Default::default(),
      signals,
      fence,
    })
  }

  pub fn wait_for(&mut self, sig: vk::Semaphore, stage: vk::ShaderStageFlags) {
    self.wait_signals.push(sig);
    self.wait_stages.push(stage);
  }

  pub fn push(&mut self, stream: Stream) {
    vk::EndCommandBuffer(stream.buffer);
    self.buffers.push(stream.buffer);
    self.streams.push(stream);
  }

  pub fn submit(&mut self, queue: vk::Queue) -> vk::Semaphore {
    let info = vk::SubmitInfo {
      sType: vk::STRUCTURE_TYPE_SUBMIT_INFO,
      pNext: std::ptr::null(),
      commandBufferCount: self.buffers.len() as u32,
      pCommandBuffers: self.buffers.as_ptr(),

      waitSemaphoreCount: self.wait_signals.len() as u32,
      pWaitSemaphores: self.wait_signals.as_ptr(),
      pWaitDstStageMask: self.wait_stages.as_ptr(),

      signalSemaphoreCount: 1,
      pSignalSemaphores: &self.signals,
    };

    vk::ResetFences(self.device, 1, &self.fence);
    vk::QueueSubmit(queue, 1, &info, self.fence);

    self.signals
  }

  pub fn sync(&mut self) -> Result<(), Error> {
    vk_check!(vk::WaitForFences(self.device, 1, &self.fence, vk::TRUE, u64::max_value()))
      .map_err(|e| Error::SyncFailed(e))
      .map(|_| ())?;

    self.clear();

    Ok(())
  }

  pub fn clear(&mut self) {
    while let Some(stream) = self.streams.pop() {
      stream.waive();
    }

    self.streams.clear();
    self.buffers.clear();
    self.wait_signals.clear();
    self.wait_stages.clear();
  }
}

impl BatchSubmit {
  /// Creates a new batch [with capacity = 1](struct.BatchSubmit.html#method.with_capacity)
  pub fn new(device: vk::Device) -> Result<Self, Error> {
    Ok(Self {
      batch: Batch::new(device)?,
    })
  }

  /// Creates a new batch
  ///
  /// Initalizes vectors to hold the streams with `capacity`.
  /// This might be useful if we know the maximum number of stream that are submited to the batch up front, because we do not need to resize and reallocate vectors as much.
  ///
  /// For the same reason we should e.g. not create a new batch every frame but use an
  /// [AutoBatch](struct.AutoBatch.html) or [Frame](struct.Frame.html) that lives outside of the render loop.
  /// Otherwise we would free the memory for the cached streams after every loop and reallocate them in the next frame again.
  pub fn with_capacity(device: vk::Device, capacity: usize) -> Result<Self, Error> {
    Ok(Self {
      batch: Batch::with_capacity(device, capacity)?,
    })
  }

  /// Waits for `sig` in `stage`
  ///
  /// Adds the semaphore to the waiting signals. Can be called multiple times, the execution will then wait on all submitted semaphores.
  pub fn wait_for(mut self, sig: vk::Semaphore, stage: vk::ShaderStageFlags) -> Self {
    self.batch.wait_for(sig, stage);
    self
  }

  /// Adds a stream to the batch
  ///
  /// After this, the stream can not be modified any more.
  /// Pushing a stream into the batch will automatically call `vk::EndCommandBuffer` on the streams command buffer.
  pub fn push(mut self, stream: Stream) -> Self {
    self.batch.push(stream);
    self
  }

  /// Submit all streams to the queue.
  ///
  /// ## Returns
  /// A tuple with
  ///  - the [BatchWait](struct.BatchWait.html) used for syncronisation
  ///  - the semaphore that indicates when the batch is done with its execution
  pub fn submit(mut self, queue: vk::Queue) -> (BatchWait, vk::Semaphore) {
    let sig = self.batch.submit(queue);
    (BatchWait { batch: self.batch }, sig)
  }

  /// Submits all streams to the queue and [syncs](struct.BatchWait.html#method.sync) with the CPU
  pub fn submit_immediate(self, queue: vk::Queue) -> Result<BatchSubmit, Error> {
    self.submit(queue).0.sync()
  }

  /// Returns all stream to the pool with [waive](struct.Stream.html#method.waive) and clears all wait signals
  ///
  /// If no buffers have been [pushed](struct.BatchSubmit.html#method.push) this as o NOP. Otherwise no work is submitted for any of the streams in the batch.
  pub fn clear(mut self) -> Self {
    self.batch.clear();
    self
  }
}

impl BatchWait {
  /// Waits in the current thread until all work in this batch is completed
  ///
  /// Yields a [BatchSubmit](struct.BatchSubmit.html) that is ready for configuration again.
  pub fn sync(mut self) -> Result<BatchSubmit, Error> {
    self.batch.sync().map(|_| BatchSubmit { batch: self.batch })
  }
}

/// Convenietly ping pong between [BatchSubmit](struct.BatchSubmit.html) and [BatchWait](struct.BatchWait.html).
///
/// Implements the same functions as BatchSubmit and BatchWait, without the need for managing both types.
/// The main difference is, that the AutoBatch is either in a submitting state or a waiting state, which is otherwise handles by the two different types.
/// Doing this with two types has the benefit, that the compiler prevents us from colling a function that is not defined for the current state.
/// However it can be tedious to manage the two types, hence the AutoBatch.
///
/// Note that in the AutoBatch nothing prevents us from calling [push](struct.AutoBatch.html#method.push) directly after [submit](struct.AutoBatch.html#method.submit),
/// which is usually not defined, because the batch was already in a waiting state. In general all functions behave exactly as in
/// [BatchSubmit](struct.BatchSubmit.html) and [BatchWait](struct.BatchWait.html) as long as they are called in their respective state, otherwise they become NOPs.
///
/// If the batch [is submitting](struct.AutoBatch.html#method.is_submitting), we can call
///  - [wait_for](struct.AutoBatch.html#method.wait_for)
///  - [push](struct.AutoBatch.html#method.push)
///  - [submit](struct.AutoBatch.html#method.submit)
///  - [submit_immediate](struct.AutoBatch.html#method.submit_immediate)
///
/// and all other methods will become NOPs. If the batch [is waiting](struct.AutoBatch.html#method.is_waiting), we can call
///  - [sync](struct.AutoBatch.html#method.wait_for)
///
/// and all other methods will become NOPs.
pub struct AutoBatch {
  submit: Option<BatchSubmit>,
  wait: Option<BatchWait>,
}

impl AutoBatch {
  /// Create a new batch [with capacity = 1](struct.AutoBatch.html#method.with_capacity)
  pub fn new(device: vk::Device) -> Result<Self, Error> {
    Ok(Self {
      submit: Some(BatchSubmit::new(device)?),
      wait: None,
    })
  }

  /// Create a new batch with `capacity` as in [BatchSubmit::with_capacity](struct.BatchSubmit.html#method.with_capacity)
  pub fn with_capacity(device: vk::Device, capacity: usize) -> Result<Self, Error> {
    Ok(Self {
      submit: Some(BatchSubmit::with_capacity(device, capacity)?),
      wait: None,
    })
  }

  /// Checks if the batch is in submitting state
  pub fn is_submitting(&self) -> bool {
    self.submit.is_some()
  }

  /// Checks if the batch is in waiting state
  pub fn is_waiting(&self) -> bool {
    self.wait.is_some()
  }

  /// Waits for `sig` in `stage`
  ///
  /// See (BatchSubmit::wait_for)[struct.BatchSubmit.html#method.wait_for].
  ///
  /// If this batch is in waiting state this function will become a NOP.
  pub fn wait_for(&mut self, sig: vk::Semaphore, stage: vk::ShaderStageFlags) -> &mut Self {
    self.submit = self.submit.take().and_then(|b| Some(b.wait_for(sig, stage)));
    self
  }

  /// Adds a stream to the batch
  ///
  /// See (BatchSubmit::push)[struct.BatchSubmit.html#method.push].
  ///
  /// If this batch is in waiting state this function will become a NOP.
  pub fn push(&mut self, stream: Stream) -> &mut Self {
    self.submit = self.submit.take().and_then(|b| Some(b.push(stream)));
    self
  }

  /// Submit all streams to the queue.
  ///
  /// See (BatchSubmit::submit)[struct.BatchSubmit.html#method.submit].
  ///
  /// If this batch is in waiting state this function will become a NOP.
  pub fn submit(&mut self, queue: vk::Queue) -> (&mut Self, Option<vk::Semaphore>) {
    match self.submit.take() {
      Some(b) => {
        let (wait, sig) = b.submit(queue);
        self.wait = Some(wait);
        (self, Some(sig))
      }
      None => (self, None),
    }
  }

  /// Submits all streams to the queue and [syncs](struct.BatchWait.html#method.sync) with the CPU
  ///
  /// If this batch is in waiting state this function will become a NOP.
  pub fn submit_immediate(&mut self, queue: vk::Queue) -> Result<&mut Self, Error> {
    self.submit(queue);
    self.sync()
  }

  /// Returns all stream to the pool with [waive](struct.Stream.html#method.waive) and clears all wait signals
  ///
  /// See [BatchWait::sync](struct.BatchWait.html#method.sync).
  ///
  /// If this batch is in submitting state this function will become a NOP.
  pub fn sync(&mut self) -> Result<&mut Self, Error> {
    match self.wait.take() {
      Some(b) => self.submit = Some(b.sync()?),
      None => (),
    }
    Ok(self)
  }
}

/// Convinietly schedule [AutoBatches](struct.AutoBatch.html) in round robin
///
/// When rendering we can build a batch in parallel while still executing the batch from one (or more) frames before.
/// Here we schedule batches in round robin with N batches. This means we can build/execute up to N batches in parallel.
///
/// We advance the frame with [next](struct.Frame.html#method.next), which syncs with the batch that has been submitted the longest time ago.
pub struct Frame {
  batches: Vec<AutoBatch>,
  index: usize,
}

impl Frame {
  /// Creates batches for `num_frames`
  ///
  /// Batches are created [with capacity = 1](struct.Frame.html#method.with_capacity)
  pub fn new(device: vk::Device, num_frames: usize) -> Result<Self, Error> {
    Self::with_capacity(device, num_frames, 1)
  }

  /// Creates batches for `num_frames`
  ///
  /// Batches are created with `batch_capacity` [BatchSubmit::with_capacity](struct.BatchSubmit.html#method.with_capacity)
  pub fn with_capacity(device: vk::Device, num_frames: usize, batch_capacity: usize) -> Result<Self, Error> {
    let mut batches = Vec::with_capacity(num_frames);
    for _i in 0..num_frames {
      batches.push(AutoBatch::with_capacity(device, batch_capacity)?);
    }

    Ok(Self { batches, index: 0 })
  }

  /// Advances the frame index and returns it
  ///
  /// The returned index is assured to be in `[0..num_frames]`.
  /// The index is incremented and moduloed with `num_frames`.
  /// [Syncs](struct.BatchWait.html#methad.sync) with the next batch that is configured.
  pub fn next(&mut self) -> Result<usize, Error> {
    self.index = (self.index + 1) % self.batches.len();
    self.batches[self.index].sync()?;
    Ok(self.index)
  }

  /// Waits for `sig` in `stage`
  ///
  /// See (AutoBatch::wait_for)[struct.BatchSubmit.html#method.wait_for].
  ///
  /// If this batch is in waiting state this function will become a NOP.
  pub fn wait_for(&mut self, sig: vk::Semaphore, stage: vk::ShaderStageFlags) -> &mut Self {
    self.batches[self.index].wait_for(sig, stage);
    self
  }

  /// Adds a stream to the batch
  ///
  /// See (BatchSubmit::push)[struct.BatchSubmit.html#method.push].
  ///
  /// If this batch is in waiting state this function will become a NOP.
  pub fn push(&mut self, stream: Stream) -> &mut Self {
    self.batches[self.index].push(stream);
    self
  }

  /// Submit all streams to the queue.
  ///
  /// See (BatchSubmit::submit)[struct.BatchSubmit.html#method.submit].
  ///
  /// If this batch is in waiting state this function will become a NOP.
  pub fn submit(&mut self, queue: vk::Queue) -> (&mut Self, Option<vk::Semaphore>) {
    let (_, sig) = self.batches[self.index].submit(queue);
    (self, sig)
  }
}
