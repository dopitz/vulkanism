//! Handle vulkan commands, command pools, command buffers and batched submitting and syncronisation
//!
//! This module's interface is mainly based on three units
//!  - [Pool](struct.Pool.html) - manages creation and reusing of command buffers in the command pool
//!  - [BatchSubmit](struct.BatchSubmit.html) and [BatchWait](struct.Pool.html) - manage submission and syncronisation of comand buffers
//!     - [AutoBatch](struct.AutoBatch.html) - conveniently handles switching between states (submitting streams to the batch, and waiting for its completion)
//!     - [Frame](struct.Frame.html) - lets you iterate through a vector of `AutoBatch` round robin style
//!  - [Stream](struct.Stream.html) - manages a command buffer, basically a builder pattern for `vk::Cmd*` commands
//!
//! The module [commands](commands/index.html) defines builder patterns for vulkan structs and commands related to `vk::Cmd*`
mod batch;
mod pool;
mod stream;

pub mod commands;

pub use batch::BatchSubmit;
pub use batch::BatchWait;
pub use batch::AutoBatch;
pub use batch::Frame;
pub use pool::Pool;
pub use stream::Stream;

#[derive(Debug, Clone)]
pub enum Error {
  InvalidQueueFamily,
  CreatePoolFailed(vk::Error),
  CreateBatchFailed(vk::Error),
  CreateStreamFailed(vk::Error),

  BeginCommandBufferFailed(vk::Error),

  SubmitFailed(vk::Error),
  SyncFailed(vk::Error),
}
