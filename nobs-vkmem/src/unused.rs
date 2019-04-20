use std::sync::Arc;
use std::sync::Mutex;

use crate::Allocator;

struct UnusedResourcesImpl {
  ring_buffer: Vec<Vec<u64>>,
  ring_index: usize,
}

#[derive(Clone)]
pub struct UnusedResources {
  imp: Arc<Mutex<UnusedResourcesImpl>>,
}

impl UnusedResources {
  pub fn new(inflight: usize) -> Self {
    let mut ring_buffer= Vec::with_capacity(inflight);
    ring_buffer.resize_with(inflight, || Default::default());
    Self {
      imp: Arc::new(Mutex::new(UnusedResourcesImpl {
        ring_buffer,
        ring_index: 0,
      })),
    }
  }

  pub fn push(&self, resource: u64) {
    let mut imp = self.imp.lock().unwrap();
    let i = imp.ring_index;
    imp.ring_buffer[i].push(resource);
  }

  pub fn free(&self, mut alloc: Allocator) -> bool {
    let mut imp = self.imp.lock().unwrap();
    let i = (imp.ring_index + imp.ring_buffer.len() - 1) % imp.ring_buffer.len();
    let free = !imp.ring_buffer[i].is_empty();

    if !imp.ring_buffer[i].is_empty() {
      alloc.destroy_many(&imp.ring_buffer[i]);
      imp.ring_buffer[i].clear();
    }

    imp.ring_index = (imp.ring_index + 1) % imp.ring_buffer.len();

    free
  }
}
