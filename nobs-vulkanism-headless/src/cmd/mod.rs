//! Handle vulkan commands, command pools and command buffers
//!
//! This modules main interface is the command [Pool](struct.Pool.html) and [commands](commands/index.html).
mod batch;
mod pool;
mod stream;

pub mod commands;

pub use pool::Pool;
pub use stream::Stream;

pub use batch::PoolB;
pub use batch::Batch;


#[derive(Debug, Clone)]
pub enum Error {
  InvalidQueueFamily,
  CreatePoolFailed(vk::Error),
  CreateStreamFailed(vk::Error),
  BeginCommandBufferFailed(vk::Error),

  SubmitFailed(vk::Error),
}
