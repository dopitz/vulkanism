mod pool;
mod stream;
mod commands;

pub use pool::Pool;
pub use stream::Stream;
pub use commands::*;

#[derive(Debug, Clone)]
pub enum Error {
  InvalidQueueFamily,
  CreateStreamFailed(vk::Error),
  BeginCommandBufferFailed(vk::Error),
}
