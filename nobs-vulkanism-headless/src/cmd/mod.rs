//! Handle vulkan commands, command pools, command buffers and batched submitting and syncronisation
//!
//! - The [CmdPool](struct.CmdPool.html) manages the creation and reusing of command buffers in the command pool
//!   [CmdPool](struct.CmdPool.html) can be used in a concurrent environment, hazards are handled internally (when a [CmdBuffer](struct.CmdBuffer.html) is either created or submitted).
//!
//! - [Stream](struct.Stream.html) and [StreamMut](struct.StreamMut.html) provide interfaces for pushing commands into a [CmdBuffer](struct.CmdBuffer.html)
//!
//! - [BatchSubmit](struct.BatchSubmit.html), [AutoBatch](struct.AutoBatch.html), [RRBatch](struct.RRBatch.html) conveniently handles submission of multiple CmdBuffers at once.
//!   Submission of command batches is an overhead heavy operation. The different batches let one submit multiple command buffers with a single call. Different flavours are offered:
//!    - [BatchSubmit](struct.BatchSubmit.html) and [BatchWait](struct.BatchWait.html) - for single submission and then waiting for completion
//!    - [AutoBatch](struct.AutoBatch.html) - multiple submissions, maximum 1 submission call is in flight
//!    - [RRBatch](struct.RRBatch.html) - multiple submissions, maximum N submission calls in flight, waiting in a round robin manner
//!
//! - The module [commands](commands/index.html) defines builder patterns for vulkan structs and commands related to `vk::cmd*`
mod batch;
mod buffer;
mod pool;

pub mod stream;
pub mod commands;

pub use batch::AutoBatch;
pub use batch::BatchSubmit;
pub use batch::BatchWait;
pub use batch::RRBatch;
pub use buffer::CmdBuffer;
pub use pool::CmdPool;

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
