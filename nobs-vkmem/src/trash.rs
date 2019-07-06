use std::sync::Arc;
use std::sync::Mutex;

use crate::Allocator;
use crate::Handle;

struct TrashImpl {
  alloc: Allocator,
  ring_buffer: Vec<Vec<Handle<u64>>>,
  ring_index: usize,
}

/// Keeps track of unused resources and deletes them
///
/// In rendering scenarios we often run into a situation that a buffer/image is not used any more, but deleting it immediately is not possible, because it is still used in a draw/dispatch operation.
/// The Trash will collect such resources and delete them when it is safe to do so and in bulk, which makes it also more efficient for the Allocator to update the memory page tables.
///
/// The main idea is to call [clean](struct.Trash.html#method.clean) once every frame. Dependeing on the number of frames that can be in flight we wait N frames befor deleting the resources.
///
/// The Trash can be used concurrently, hazards are resolved internally.
#[derive(Clone)]
pub struct Trash {
  imp: Arc<Mutex<TrashImpl>>,
}

impl Trash {
  /// Create a Trash for the specified Allocator
  ///
  /// # Arguments
  /// * `alloc` - the Alloctator that will be used to destroy the tracked resources
  /// * `inflight` - number of frames that may be computed simultaneously, must be larger than 1
  ///
  /// # Returns
  /// The Trash object
  pub fn new(alloc: Allocator, inflight: usize) -> Self {
    let mut ring_buffer = Vec::with_capacity(inflight);
    ring_buffer.resize_with(inflight, || Default::default());
    Self {
      imp: Arc::new(Mutex::new(TrashImpl {
        alloc,
        ring_buffer,
        ring_index: 0,
      })),
    }
  }

  /// Mark the given resource for deletion
  pub fn push(&self, resource: Handle<u64>) {
    let mut imp = self.imp.lock().unwrap();
    let i = imp.ring_index;
    imp.ring_buffer[i].push(resource);
  }
  /// Mark the specified image for deletion
  pub fn push_image(&self, img: vk::Image) {
    self.push(Handle::Image(img));
  }
  /// Mark the specified buffer for deletion
  pub fn push_buffer(&self, buf: vk::Buffer) {
    self.push(Handle::Buffer(buf));
  }

  /// Cleans up resources that can be deleted safely
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
