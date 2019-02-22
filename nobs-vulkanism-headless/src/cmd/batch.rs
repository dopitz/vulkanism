use std::sync::Arc;
use std::sync::Condvar;
use std::sync::Mutex;

use vk;

use crate::cmd::batch;
use crate::cmd::stream;
use crate::cmd::Error;

type SynBatches = Arc<(Mutex<Batches>, Condvar)>;

pub struct PoolB {
  device: vk::Device,
  queue_family: u32,
  handle: vk::CommandPool,

  batches: SynBatches,
}

impl PoolB {
  pub fn new(device: vk::Device, queue_family: u32, num_batches: usize) -> Result<Self, Error> {
    let info = vk::CommandPoolCreateInfo {
      sType: vk::STRUCTURE_TYPE_COMMAND_POOL_CREATE_INFO,
      pNext: std::ptr::null(),
      flags: vk::COMMAND_POOL_CREATE_RESET_COMMAND_BUFFER_BIT,
      queueFamilyIndex: queue_family,
    };

    let mut handle = vk::NULL_HANDLE;
    vk_check!(vk::CreateCommandPool(device, &info, std::ptr::null(), &mut handle)).map_err(|e| Error::CreatePoolFailed(e))?;

    Ok(Self {
      device,
      queue_family,
      handle,

      batches: Arc::new((Mutex::new(Batches::new(num_batches)), Condvar::new())),
    })
  }

  pub fn next_batch(&mut self) -> Batch {
    let mut index = 0;

    let &(ref mtx, ref cv) = &*self.batches;
    let mut batches = mtx.lock().unwrap();

    loop {
      batches = cv.wait(batches).unwrap();

      if let Some(i) = batches.used.pop() {
        index = i;
        break;
      }
    }

    Batch {
      batches: Arc::clone(&self.batches),
      index,
    }
  }
}

struct Batches {
  pub batches: Vec<(Vec<Stream>, Vec<vk::Fence>)>,
  pub used: Vec<usize>,
}

impl Batches {
  fn new(num_batches: usize) -> Self {
    let mut batches = Vec::with_capacity(num_batches);
    let mut used = Vec::with_capacity(num_batches);
    for i in 0..num_batches {
      batches.push(Default::default());
      used.push(i);
    }

    Self { batches, used }
  }
}

pub struct Batch {
  batches: SynBatches,
  index: usize,
}

pub impl Batch {
  pub fn submit()
}


pub struct Stream {}
