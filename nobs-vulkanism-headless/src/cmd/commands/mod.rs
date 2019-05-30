mod barrier;
mod blit;
mod copy;
mod dispatch;
mod draw;
mod pipeline;
mod renderpass;

pub use barrier::*;
pub use blit::*;
pub use copy::*;
pub use dispatch::*;
pub use draw::*;
pub use pipeline::*;
pub use renderpass::*;

use super::Stream;

/// Allows to use [push](../stream/struct.Stream.html#method.push) on a [Stream](../stream/struct.Stream.html)
pub trait StreamPush {
  fn enqueue(&self, cs: Stream) -> Stream;
}

pub trait StreamPushMut {
  fn enqueue_mut(&mut self, cs: Stream) -> Stream;
}
