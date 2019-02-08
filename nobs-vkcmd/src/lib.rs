#[macro_use]
extern crate nobs_vk as vk;

mod pool;
mod stream;
mod streampush;

pub use self::pool::Pool;
pub use self::stream::Stream;
pub use self::streampush::*;

#[derive(Debug, Clone)]
pub enum Error {
  InvalidQueueFamily,
  CreateStreamFailed(vk::Error),
  BeginCommandBufferFailed(vk::Error),
}
