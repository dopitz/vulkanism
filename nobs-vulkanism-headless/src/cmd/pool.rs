use std::sync::Arc;
use std::sync::Mutex;

use vk;

use crate::cmd::stream::StreamCache;
use crate::cmd::Error;
use crate::cmd::Stream;

/// Wrapps a vulkan command pool
///
/// The Pool is basically a chach for [Streams](struct.Stream.html).
/// The only method [begin_stream](struct.Pool.html#method.begin_stream) either creates a new stream or reuses one that is currently not unused.
#[derive(Clone)]
pub struct Pool {
  pub device: vk::Device,
  pub queue_family: u32,
  pub handle: vk::CommandPool,

  streams: Arc<Mutex<StreamCache>>,
}

impl Drop for Pool {
  fn drop(&mut self) {
    let mut streams = self.streams.lock().unwrap();
    if Arc::strong_count(&self.streams) == 1 {
      vk::DestroyCommandPool(self.device, self.handle, std::ptr::null());
    }
  }
}

impl Pool {
  /// Create a new command pool
  ///
  /// ## Arguments
  ///  * `device` - vulkan device handle
  ///  * `queue_family` - index of the queue family to which the commands are submitted
  pub fn new(device: vk::Device, queue_family: u32) -> Result<Self, vk::Error> {
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
      streams: Arc::new(Mutex::new(StreamCache::new(device, handle))),
    })
  }

  /// Get a new stream
  ///
  /// Reuses an unused command buffer if there is one, if not creates a new one.
  /// Calles [begin](struct.Stream.html#method.begin) on the stream and returns it.
  pub fn begin_stream(&self) -> Result<Stream, Error> {
    Stream::new(self.streams.clone()).and_then(|s| s.begin())
  }
}
