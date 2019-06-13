use super::buffer::BufferCache;
use super::Error;
use super::CmdBuffer;
use std::sync::Arc;
use std::sync::Mutex;
use vk;

/// Wrapps a vulkan command pool
///
/// The Pool is basically a chache for [command buffers](struct.CmdBuffer.html).
/// The only method [begin_stream](struct.Pool.html#method.begin_stream) either creates a new stream or reuses one from the cache.
#[derive(Clone)]
pub struct CmdPool {
  pub device: vk::Device,
  pub queue_family: u32,
  pub handle: vk::CommandPool,

  buffers: Arc<Mutex<BufferCache>>,
}

impl CmdPool {
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
      buffers: Arc::new(Mutex::new(BufferCache::new(device, handle))),
    })
  }

  /// Get a new stream
  ///
  /// Reuses an unused command buffer if there is one, if not creates a new one.
  /// Calles [begin](struct.CmdBuffer.html#method.begin) on the stream and returns it.
  pub fn begin_stream(&self) -> Result<CmdBuffer, Error> {
    CmdBuffer::new(self.buffers.clone()).and_then(|s| s.begin())
  }
}
