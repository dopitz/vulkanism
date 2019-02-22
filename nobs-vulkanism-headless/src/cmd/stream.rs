use std::rc::Rc;
use std::rc::Weak as RcWeak;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::Weak;

use vk;

use crate::cmd::commands::StreamPush;
use crate::cmd::pool;
use crate::cmd::Error;

pub struct Pool {
  device: vk::Device,
  device_extensions: Rc<vk::DeviceExtensions>,
  pool: vk::CommandPool,
  unused: Vec<Stream>,

  group_index: usize,
  groups: Vec<SubmitGroup>,
}

#[derive(Default)]
struct SubmitGroup {
  streams: Vec<Stream>,
  fences: Vec<vk::Fence>,
}

impl Pool {
  pub fn new(device: vk::Device, pool: vk::CommandPool, inflight: usize) -> Arc<Mutex<Self>> {
    let unused = Vec::new();
    let mut groups = Vec::with_capacity(inflight);
    for _ in 0..inflight {
      groups.push(Default::default());
    }

    Arc::new(Mutex::new(Self {
      device,
      device_extensions: Rc::new(vk::DeviceExtensions::new(device)),
      pool,
      unused,
      group_index: 0,
      groups,
    }))
  }

  pub fn stream_begin(streams: &mut Arc<Mutex<Self>>, queue: vk::device::Queue) -> Result<Stream, Error> {
    let weak = Arc::downgrade(streams);
    let mut streams = streams.lock().unwrap();
    let mut stream = match streams.unused.pop() {
      Some(s) => s,
      None => {
        let device = streams.device;
        let mut err = None;

        let buffer = {
          let info = vk::CommandBufferAllocateInfo {
            sType: vk::STRUCTURE_TYPE_COMMAND_BUFFER_ALLOCATE_INFO,
            pNext: std::ptr::null(),
            level: vk::COMMAND_BUFFER_LEVEL_PRIMARY,
            commandPool: streams.pool,
            commandBufferCount: 1,
          };
          let mut h = vk::NULL_HANDLE;
          err = err.and(
            vk_check!(vk::AllocateCommandBuffers(device, &info, &mut h))
              .map_err(|e| Error::CreateStreamFailed(e))
              .err(),
          );
          h
        };

        let fence = {
          let info = vk::FenceCreateInfo {
            sType: vk::STRUCTURE_TYPE_FENCE_CREATE_INFO,
            pNext: std::ptr::null(),
            flags: vk::FENCE_CREATE_SIGNALED_BIT,
          };
          let mut h = vk::NULL_HANDLE;
          err = err.and(
            vk_check!(vk::CreateFence(device, &info, std::ptr::null(), &mut h))
              .map_err(|e| Error::CreateStreamFailed(e))
              .err(),
          );
          h
        };

        let signals = {
          let info = vk::SemaphoreCreateInfo {
            sType: vk::STRUCTURE_TYPE_SEMAPHORE_CREATE_INFO,
            pNext: std::ptr::null(),
            flags: 0,
          };
          let mut h = vk::NULL_HANDLE;
          err = err.and(
            vk_check!(vk::CreateSemaphore(device, &info, std::ptr::null(), &mut h))
              .map_err(|e| Error::CreateStreamFailed(e))
              .err(),
          );
          h
        };

        Stream {
          device,
          device_extensions: Rc::clone(&streams.device_extensions),
          queue,
          wait_signals: Default::default(),
          wait_stages: Default::default(),
          buffer,
          signals,
          fence,
          streams: weak,

          err,
        }
      }
    };

    stream.wait_signals.clear();
    stream.wait_stages.clear();
    vk::ResetCommandBuffer(stream.buffer, 0);

    let info = vk::CommandBufferBeginInfo {
      sType: vk::STRUCTURE_TYPE_COMMAND_BUFFER_BEGIN_INFO,
      pNext: std::ptr::null(),
      flags: vk::COMMAND_BUFFER_USAGE_ONE_TIME_SUBMIT_BIT,
      pInheritanceInfo: std::ptr::null(),
    };
    vk_check!(vk::BeginCommandBuffer(stream.buffer, &info)).map_err(|e| Error::BeginCommandBufferFailed(e))?;

    Ok(stream)
  }

  pub fn retain(&mut self, stream: Stream) {
    self.unused.push(stream);
  }

  pub fn submit(&mut self, stream: Stream) {
    let g = &mut self.groups[self.group_index];
    g.fences.push(stream.fence);
    g.streams.push(stream);
  }

  pub fn next_frame(&mut self) -> usize {
    self.group_index = (self.group_index + 1) % self.groups.len();
    let g = &mut self.groups[self.group_index];

    // wait until the group is finished
    if !g.fences.is_empty() {
      vk_uncheck!(vk::WaitForFences(
        self.device,
        g.fences.len() as u32,
        g.fences.as_ptr(),
        vk::TRUE,
        u64::max_value()
      ));
    }

    // put streams back into unused buffer
    while let Some(s) = g.streams.pop() {
      self.unused.push(s);
    }

    g.fences.clear();
    g.streams.clear();
    self.group_index as usize
  }

  pub fn wait_all(&mut self) {
    for _i in 0..self.groups.len() {
      self.next_frame();
    }
  }
}

/// Manages a vulkan command buffer
///
/// The command stream handles the life cycle of a command buffer.
/// It has
pub struct Stream {
  pub device: vk::Device,
  pub device_extensions: Rc<vk::DeviceExtensions>,
  pub queue: vk::device::Queue,
  pub buffer: vk::CommandBuffer,
  wait_signals: Vec<vk::Semaphore>,
  wait_stages: Vec<vk::ShaderStageFlags>,
  signals: vk::Semaphore,
  fence: vk::Fence,
  streams: Weak<Mutex<Pool>>,

  err: Option<Error>,
}

impl Drop for Stream {
  fn drop(&mut self) {
    vk::DestroySemaphore(self.device, self.signals, std::ptr::null());
    vk::DestroyFence(self.device, self.fence, std::ptr::null());

    if let Some(streams) = self.streams.upgrade() {
      if let Ok(streams) = streams.lock() {
        vk::FreeCommandBuffers(self.device, streams.pool, 1, &self.buffer);
      }
    }
  }
}

impl Stream {
  pub fn map_err(mut self, err: Result<(), Error>) -> Self {
    match err {
      Ok(_) => (),
      Err(e) => self.err = Some(e),
    }
    self
  }

  pub fn push<T: StreamPush>(self, o: &T) -> Self {
    match self.err {
      None => o.enqueue(self),
      Some(_) => self,
    }
  }

  pub fn wait_for(mut self, sig: vk::Semaphore, stage: vk::ShaderStageFlags) -> Self {
    if self.err.is_none() {
      self.wait_signals.push(sig);
      self.wait_stages.push(stage);
    }
    self
  }

  fn submit_inner(self, signal: bool, immediate: bool) -> Result<vk::Semaphore, Error> {
    vk::EndCommandBuffer(self.buffer);

    let info = vk::SubmitInfo {
      sType: vk::STRUCTURE_TYPE_SUBMIT_INFO,
      pNext: std::ptr::null(),
      commandBufferCount: 1,
      pCommandBuffers: &self.buffer,

      waitSemaphoreCount: self.wait_signals.len() as u32,
      pWaitSemaphores: self.wait_signals.as_ptr(),
      pWaitDstStageMask: self.wait_stages.as_ptr(),

      signalSemaphoreCount: match signal {
        true => 1,
        false => 0,
      },
      pSignalSemaphores: match signal {
        true => &self.signals,
        false => std::ptr::null(),
      },
    };

    vk::ResetFences(self.device, 1, &self.fence);
    vk::QueueSubmit(self.queue.handle, 1, &info, self.fence);

    let sig = self.signals;

    if let Some(streams) = self.streams.upgrade() {
      let mut streams = streams.lock().unwrap();
      if immediate {
        vk_check!(vk::WaitForFences(self.device, 1, &self.fence, vk::TRUE, u64::max_value())).map_err(|e| Error::SubmitFailed(e))?;
        streams.retain(self);
      } else {
        streams.submit(self);
      }
    }

    Ok(sig)
  }

  pub fn submit(self) -> Result<(), Error> {
    self.submit_inner(false, false).map(|sig| ())
  }

  pub fn submit_signals(self) -> Result<vk::Semaphore, Error> {
    self.submit_inner(true, false)
  }

  pub fn submit_immediate(self) -> Result<(), Error> {
    self.submit_inner(false, true).map(|sig| ())
  }
}
