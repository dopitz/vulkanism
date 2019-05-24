use std::sync::Arc;
use std::sync::Mutex;

use crate::Allocator;

struct TrashImpl {
  alloc: Allocator,
  ring_buffer: Vec<Vec<u64>>,
  ring_index: usize,
}

#[derive(Clone)]
pub struct Trash {
  imp: Arc<Mutex<TrashImpl>>,
}

impl Trash {
  pub fn new(alloc: Allocator, inflight: usize) -> Self {
    let mut ring_buffer= Vec::with_capacity(inflight);
    ring_buffer.resize_with(inflight, || Default::default());
    Self {
      imp: Arc::new(Mutex::new(TrashImpl {
        alloc,
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

  pub fn clean(&self) -> bool {
    let mut imp = self.imp.lock().unwrap();
    let next = (imp.ring_index + 1) % imp.ring_buffer.len();
    let free = !imp.ring_buffer[next].is_empty();

    if !imp.ring_buffer[next].is_empty() {
      imp.alloc.clone().destroy_many(&imp.ring_buffer[next]);
      imp.ring_buffer[next].clear();
    }

    imp.ring_index = next;
    free
  }
}
