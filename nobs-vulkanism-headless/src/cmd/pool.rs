use std::sync::Arc;
use std::sync::Mutex;

use vk;

use crate::cmd::stream;
use crate::cmd::batch;
use crate::cmd::Error;

/// Wrapps a vulkan command pool
///
/// The methods [begin](struct.Pool.html#method.begin) and [begin_after](struct.Pool.html#method.begin) start a new command [Stream](struct.Stream.html).
/// The new Stream automatically creates command buffers on demand and reuses them if possible.
///
/// Command streams are submitted into batches. Batches can be synchronized with [next_frame](struct.Pool.html#method.next_frame) or [wait_all](struct.Pool.html#method.wait_all).
/// After wait_all all pending commands in all batches will be synchronized with the CPU. In next_frame we wait and return as soon as there is a batch with no pending commands available.
///
/// The number of batches can be controlled with the `inflight` parameter during construction.
///
/// All methods in the command pool are thread safe. However be careful when building command streams in multiple threads and calling [next_frame](struct.Pool.html#method.next_frame) or [wait_all](struct.Pool.html#method.wait_all).
/// If either of the synchronizing methods is called before every thread has finishd building the streams and called submit on them the stream that was submitted late will be put into the next batch.
pub struct Pool {
  pub device: vk::Device,
  pub queue_family: u32,
  pub handle: vk::CommandPool,

  streams: Arc<Mutex<stream::Pool>>,
}

impl Drop for Pool {
  fn drop(&mut self) {
    self.wait_all();
    vk::DestroyCommandPool(self.device, self.handle, std::ptr::null());
  }
}

impl Pool {
  /// Create a new command pool
  ///
  /// ## Arguments
  ///  * `device` - vulkan device handle
  ///  * `queue_family` - index of the queue family to which the commands are submitted
  ///  * `inflight` - number of parallel batches with pending commands
  pub fn new(device: vk::Device, queue_family: u32, inflight: usize) -> Result<Self, vk::Error> {
    let pool_info = vk::CommandPoolCreateInfo {
      sType: vk::STRUCTURE_TYPE_COMMAND_POOL_CREATE_INFO,
      pNext: std::ptr::null(),
      flags: vk::COMMAND_POOL_CREATE_RESET_COMMAND_BUFFER_BIT,
      queueFamilyIndex: queue_family,
    };

    let mut handle = vk::NULL_HANDLE;
    vk_check!(vk::CreateCommandPool(device, &pool_info, std::ptr::null(), &mut handle))?;
    Ok(Self {
      device,
      queue_family,
      handle,
      streams: stream::Pool::new(device, handle, inflight),
    })
  }

  /// Begins a new command [Stream](struct.Stream.html).
  ///
  /// Creates a new command buffer for the stream if necessary, otherwise reuses one that has been created before.
  /// The returned command stream is ready to receive commands.
  ///
  /// ## Arguments
  ///  * `queue` - vulkan queue handle to submit to
  pub fn begin(&mut self, queue: vk::device::Queue) -> Result<stream::Stream, Error> {
    if queue.family != self.queue_family {
      Err(Error::InvalidQueueFamily)?
    }
    stream::Pool::stream_begin(&mut self.streams, queue)
  }

  /// Begins a new command [Stream](struct.Stream.html).
  ///
  /// Same as [begin](struct.Pool.html#method.begin) but also waits for `sig` before the commands in this stream are started.
  ///
  /// ## Arguments
  ///  * `queue` - vulkan queue handle to submit to
  ///  * `sig` - vulkan semaphore handle that needs to be signaled before the execution of this command stream begins
  ///  * `stage` - stage at which the semaphore wait will occure
  pub fn begin_after(
    &mut self,
    queue: vk::device::Queue,
    sig: vk::Semaphore,
    stage: vk::ShaderStageFlags,
  ) -> Result<stream::Stream, Error> {
    self.begin(queue).map(|cs| cs.wait_for(sig, stage))
  }

  /// Waits until the next batch of command streams is ready again
  ///
  /// Synchronizes commands with the CPU.
  ///
  /// ## Returns
  /// The index of the command batch that is ready, inside `[0, num_batches - 1]`
  pub fn next_frame(&mut self) -> usize {
    self.streams.lock().unwrap().next_frame()
  }

  /// Waits until all pending commands are finished.
  pub fn wait_all(&mut self) {
    self.streams.lock().unwrap().wait_all()
  }
}
