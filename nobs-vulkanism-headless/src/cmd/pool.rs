use std::sync::Arc;
use std::sync::Mutex;

use vk;

use crate::cmd::stream;
use crate::cmd::Error;

pub struct Pool {
  pub device: vk::Device,
  pub queue_family: u32,
  pub handle: vk::CommandPool,

  streams: Arc<Mutex<stream::Pool>>,
}

impl Drop for Pool {
  fn drop(&mut self) {
    self.wait_all();
    vk::DestroyCommandPool(self.device, self.handle, std::ptr::null());
  }
}

impl Pool {
  pub fn new(device: vk::Device, queue_family: u32, inflight: usize) -> Result<Self, vk::Error> {
    let pool_info = vk::CommandPoolCreateInfo {
      sType: vk::STRUCTURE_TYPE_COMMAND_POOL_CREATE_INFO,
      pNext: std::ptr::null(),
      flags: vk::COMMAND_POOL_CREATE_RESET_COMMAND_BUFFER_BIT,
      queueFamilyIndex: queue_family,
    };

    let mut handle = vk::NULL_HANDLE;
    vk_check!(vk::CreateCommandPool(device, &pool_info, std::ptr::null(), &mut handle))?;
    Ok(Self {
      device,
      queue_family,
      handle,
      streams: stream::Pool::new(device, handle, inflight),
    })
  }

  pub fn begin(&mut self, queue: vk::device::Queue) -> Result<stream::Stream, Error> {
    if queue.family != self.queue_family {
      Err(Error::InvalidQueueFamily)?
    }
    stream::Pool::stream_begin(&mut self.streams, queue)
  }

  pub fn next_frame(&mut self) -> usize {
    self.streams.lock().unwrap().next_frame()
  }

  pub fn wait_all(&mut self) {
    self.streams.lock().unwrap().wait_all()
  }
}
