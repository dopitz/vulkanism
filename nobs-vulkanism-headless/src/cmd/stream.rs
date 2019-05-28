use std::sync::Arc;
use std::sync::Mutex;
use std::sync::Weak;

use vk;

use crate::cmd::commands::StreamPush;
use crate::cmd::Error;

pub struct StreamCache {
  device: vk::Device,
  pool: vk::CommandPool,
  streams: Vec<Stream>,
}

impl Drop for StreamCache {
  fn drop(&mut self) {
    vk::DestroyCommandPool(self.device, self.pool, std::ptr::null());
  }
}

impl StreamCache {
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
/// The stream is basically a builder pattern for all vulkan functions related to `vk::Cmd*`.
/// For every command in [commands](commands/index.html) we can use [push](struct.Stream.html#method.push) to enqueue it to the command stream.
///
/// When we are done building the command stream, we push it into a batch ([BatchSubmit](struct.BatchSubmit.html), [AutoBatch](struct.AutoBatch.html), [Frame](struct.Frame.html)).
/// The batch can then submit all streams with one queue submit call. This is useful, since queue submit calls are very overhead heavy.
pub struct Stream {
  pub buffer: vk::CommandBuffer,
  streams: Weak<Mutex<StreamCache>>,
}

impl Stream {
  /// Is only called by [Pool](struct.Pool.html)
  pub fn new(streams: Arc<Mutex<StreamCache>>) -> Result<Self, Error> {
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

  /// Is implicitly called in [Pool::begin_stream](struct.Pool.html#method.begin_stream)
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

  /// Pushes a command into a stream.
  ///
  /// Any struct implementing the [StreamPush](commands/trait.StreamPush.html) trait can be pushed into the stream.
  /// This can be used to build more complex commands from many primitive ones and be able to push them with one call.
  pub fn push<T: StreamPush>(self, o: &T) -> Self {
    o.enqueue(self)
  }
  /// Pushes a command contained in the option into a stream. NOP if the Option is None.
  ///
  /// Any struct implementing the [StreamPush](commands/trait.StreamPush.html) trait can be pushed into the stream.
  /// This can be used to build more complex commands from many primitive ones and be able to push them with one call.
  pub fn push_if<T: StreamPush>(self, o: &Option<T>) -> Self {
    match o {
      Some(c) => c.enqueue(self),
      None => self,
    }
  }

  /// Pushes a lambda into a stream.
  ///
  /// Can be used to push complex command logic into stream.
  pub fn push_fnmut<F: FnMut(Self) -> Self>(self, mut f: F) -> Self {
    f(self)
  }
  /// Pushes a lambda into a stream.
  ///
  /// Can be used to push complex command logic into stream.
  pub fn push_fn<F: Fn(Self) -> Self>(self, f: F) -> Self {
    f(self)
  }

  /// Returns the stream to the pool.
  ///
  /// This breaks up configuring the stream and returns it to the pool.
  /// No vulkan commands are submitted, the command buffer can be reused again.
  pub fn waive(self) {
    if let Some(streams) = self.streams.upgrade() {
      streams.lock().unwrap().streams.push(self)
    }
  }
}
