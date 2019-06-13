use super::stream::*;
use super::Error;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::Weak;
use vk;

pub struct BufferCache {
  device: vk::Device,
  pool: vk::CommandPool,
  streams: Vec<CmdBuffer>,
}

impl Drop for BufferCache {
  fn drop(&mut self) {
    vk::DestroyCommandPool(self.device, self.pool, std::ptr::null());
  }
}

impl BufferCache {
  pub fn new(device: vk::Device, pool: vk::CommandPool) -> Self {
    Self {
      device,
      pool,
      streams: Default::default(),
    }
  }
}

/// Wrapps a vulkan command buffer
///
/// The command buffer is basically a builder pattern for all vulkan functions related to `vk::cmd*`.
/// For every command in [commands](commands/index.html) we can use [push](struct.CmdBuffer.html#method.push) to enqueue it to the command buffer.
///
/// When we are done building the command buffer, we push it into a batch ([BatchSubmit](struct.BatchSubmit.html), [AutoBatch](struct.AutoBatch.html), [RRBatch](struct.RRBatch.html)).
/// The batch can then submit all streams with one vulkan api call. This is useful, since queue submit calls are very overhead heavy.
///
/// Letting the [CmdBuffer](struct.CmdBuffer.html) go out of scope will call [drop](struct.CmdBuffer.html#method.drop). This will redeem the CmdBuffer to the pool without subitting it.
pub struct CmdBuffer {
  pub buffer: vk::CommandBuffer,
  streams: Weak<Mutex<BufferCache>>,
}

impl Drop for CmdBuffer {
  /// Returns the stream to the pool.
  ///
  /// This breaks up configuring the stream and returns it to the pool.
  /// No vulkan commands are submitted, the command buffer can be reused again.
  fn drop(&mut self) {
    if let Some(streams) = self.streams.upgrade() {
      streams.lock().unwrap().streams.push(Self {
        buffer: self.buffer,
        streams: self.streams.clone(),
      });
    }
  }
}

impl CmdBuffer {
  /// Is only called by [CmdPool](struct.CmdPool.html)
  pub fn new(streams: Arc<Mutex<BufferCache>>) -> Result<Self, Error> {
    let (device, pool, top) = {
      let mut streams = streams.lock().unwrap();
      (streams.device, streams.pool, streams.streams.pop())
    };

    Ok(match top {
      Some(s) => s,
      None => Self {
        buffer: {
          let info = vk::CommandBufferAllocateInfo {
            sType: vk::STRUCTURE_TYPE_COMMAND_BUFFER_ALLOCATE_INFO,
            pNext: std::ptr::null(),
            level: vk::COMMAND_BUFFER_LEVEL_PRIMARY,
            commandPool: pool,
            commandBufferCount: 1,
          };
          let mut h = vk::NULL_HANDLE;
          vk_check!(vk::AllocateCommandBuffers(device, &info, &mut h)).map_err(|e| Error::CreateStreamFailed(e))?;
          h
        },
        streams: Arc::downgrade(&streams),
      },
    })
  }

  /// Is implicitly called in [CmdPool::begin_stream](struct.CmdPool.html#method.begin_stream)
  pub fn begin(self) -> Result<Self, Error> {
    vk::ResetCommandBuffer(self.buffer, 0);

    let info = vk::CommandBufferBeginInfo {
      sType: vk::STRUCTURE_TYPE_COMMAND_BUFFER_BEGIN_INFO,
      pNext: std::ptr::null(),
      flags: vk::COMMAND_BUFFER_USAGE_ONE_TIME_SUBMIT_BIT,
      pInheritanceInfo: std::ptr::null(),
    };
    vk_check!(vk::BeginCommandBuffer(self.buffer, &info)).map_err(|e| Error::BeginCommandBufferFailed(e))?;

    Ok(self)
  }
}

impl Stream for CmdBuffer {
  fn push<T: StreamPush>(self, o: &T) -> Self {
    o.enqueue(self)
  }
}

impl StreamMut for CmdBuffer {
  fn push_mut<T: StreamPushMut>(self, o: &mut T) -> Self {
    o.enqueue_mut(self)
  }
}
