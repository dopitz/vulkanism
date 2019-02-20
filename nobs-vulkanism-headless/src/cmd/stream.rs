use std::sync::Arc;
use std::sync::Mutex;
use std::sync::Weak;

use vk;

use crate::cmd::pool;
use crate::cmd::Error;
use crate::cmd::StreamPush;

pub struct Pool {
  device: vk::Device,
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

        let buffer = {
          let info = vk::CommandBufferAllocateInfo {
            sType: vk::STRUCTURE_TYPE_COMMAND_BUFFER_ALLOCATE_INFO,
            pNext: std::ptr::null(),
            level: vk::COMMAND_BUFFER_LEVEL_PRIMARY,
            commandPool: streams.pool,
            commandBufferCount: 1,
          };
          let mut h = vk::NULL_HANDLE;
          vk_check!(vk::AllocateCommandBuffers(device, &info, &mut h)).map_err(|e| Error::CreateStreamFailed(e))?;
          h
        };

        let fence = {
          let info = vk::FenceCreateInfo {
            sType: vk::STRUCTURE_TYPE_FENCE_CREATE_INFO,
            pNext: std::ptr::null(),
            flags: vk::FENCE_CREATE_SIGNALED_BIT,
          };
          let mut h = vk::NULL_HANDLE;
          vk_check!(vk::CreateFence(device, &info, std::ptr::null(), &mut h)).map_err(|e| Error::CreateStreamFailed(e))?;
          h
        };

        let signals = {
          let info = vk::SemaphoreCreateInfo {
            sType: vk::STRUCTURE_TYPE_SEMAPHORE_CREATE_INFO,
            pNext: std::ptr::null(),
            flags: 0,
          };
          let mut h = vk::NULL_HANDLE;
          vk_check!(vk::CreateSemaphore(device, &info, std::ptr::null(), &mut h)).map_err(|e| Error::CreateStreamFailed(e))?;
          h
        };

        Stream {
          device,
          queue,
          wait_signals: Default::default(),
          wait_stages: Default::default(),
          buffer,
          signals,
          fence,
          streams: weak,
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

pub struct Stream {
  device: vk::Device,
  queue: vk::device::Queue,
  wait_signals: Vec<vk::Semaphore>,
  wait_stages: Vec<vk::ShaderStageFlags>,
  buffer: vk::CommandBuffer,
  signals: vk::Semaphore,
  fence: vk::Fence,
  streams: Weak<Mutex<Pool>>,
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
  pub fn begin(pool: &mut pool::Pool, queue: vk::device::Queue) -> Result<Self, Error> {
    pool.begin(queue)
  }

  pub fn push<T: StreamPush>(self, o: &T) -> Self {
    o.enqueue(self.buffer);
    self
  }

  pub fn wait_for(mut self, sig: vk::Semaphore, stage: vk::ShaderStageFlags) -> Self {
    self.wait_signals.push(sig);
    self.wait_stages.push(stage);
    self
  }

  fn submit_inner(self, signal: bool, immediate: bool) -> vk::Semaphore {
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
        vk_uncheck!(vk::WaitForFences(self.device, 1, &self.fence, vk::TRUE, u64::max_value()));
        streams.retain(self);
      } else {
        streams.submit(self);
      }
    }

    sig
  }

  pub fn submit(self) {
    self.submit_inner(false, false);
  }

  pub fn submit_signals(self) -> vk::Semaphore {
    self.submit_inner(true, false)
  }

  pub fn submit_immediate(self) {
    self.submit_inner(false, true);
  }

  pub fn get_commandbuffer(&self) -> vk::CommandBuffer {
    self.buffer
  }
}

pub struct Sp<'a, T: StreamPush> {
  pub p: &'a T,
}

impl<'a, T: StreamPush> std::ops::Shl<Sp<'a, T>> for Stream {
  type Output = Self;

  fn shl(mut self, push: Sp<'a, T>) -> Stream {
    self.push(push.p)
  }
}
