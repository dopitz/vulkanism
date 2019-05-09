use std::sync::Arc;
use std::sync::Mutex;

use crate::Allocator;

struct ManagedResourcesImpl {
  unsused_ring_buffer: Vec<Vec<u64>>,
  unsused_ring_index: usize,
}

#[derive(Clone)]
pub struct ManagedResources {
  imp: Arc<Mutex<ManagedResourcesImpl>>,
  alloc: Arc<Allocator>,
}

impl ManagedResources {
  pub fn new(alloc: &Arc<Allocator>, inflight: usize) -> Self {
    let mut unused_ring_buffer = Vec::with_capacity(inflight);
    unused_ring_buffer.resize_with(inflight, || Default::default());
    Self {
      imp: Arc::new(Mutex::new(UnusedResourcesImpl {
        unused_ring_buffer,
        unused_ring_index: 0,
      })),
      alloc: alloc.clone(),
    }
  }

  pub fn get_allocator() -> Arc<Allocator> {
    self.allo
  }

  pub fn push(&self, resource: u64) {
    let mut imp = self.imp.lock().unwrap();
    let i = imp.unused_ring_index;
    imp.unused_ring_buffer[i].push(resource);
  }

  pub fn destroy(&self, mut alloc: Allocator) -> bool {
    let mut imp = self.imp.lock().unwrap();
    let i = (imp.unused_ring_index + imp.unused_ring_buffer.len() - 1) % imp.unused_ring_buffer.len();
    let free = !imp.unused_ring_buffer[i].is_empty();

    if !imp.unused_ring_buffer[i].is_empty() {
      alloc.destroy_many(&imp.unused_ring_buffer[i]);
      imp.unused_ring_buffer[i].clear();
    }

    imp.unused_ring_index = (imp.unused_ring_index + 1) % imp.unused_ring_buffer.len();

    free
  }
}
