mod pool;
mod stream;
mod streampush;

pub use pool::Pool;
pub use stream::Stream;
pub use streampush::*;

#[derive(Debug, Clone)]
pub enum Error {
  InvalidQueueFamily,
  CreateStreamFailed(vk::Error),
  BeginCommandBufferFailed(vk::Error),
}
