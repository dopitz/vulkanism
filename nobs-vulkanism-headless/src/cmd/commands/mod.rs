mod barrier;
mod copy;
mod dispatch;
mod draw;
mod framebuffer;
mod pipeline;

pub use barrier::*;
pub use copy::*;
pub use dispatch::*;
pub use draw::*;
pub use framebuffer::*;
pub use pipeline::*;

use super::Stream;

/// Allows to use [push](../stream/struct.Stream.html#method.push) on a [Stream](../stream/struct.Stream.html)
pub trait StreamPush {
  fn enqueue(&self, cs: Stream) -> Stream;
}
