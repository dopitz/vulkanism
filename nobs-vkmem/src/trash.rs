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
/// In rendering scenarios we often run into a situation that a buffer/image will not be used any more in the future, but deleting it immediately is not possible, because it is currently used in a draw/dispatch operation.
/// The Trash will collect such resources and delete them, when it is safe to do so and in bulk, which makes it also more efficient for the Allocator to update the memory page tables.
///
/// The main idea is to call [clean](struct.Trash.html#method.clean) once every frame. Dependeing on the number of frames that can be in flight we wait N frames before deleting the resources.
///
/// The Trash can be used concurrently, hazards are resolved internally.
///
/// The example shows the basic usage of [Trash](struct.Trash.html). 
///
/// ```rust
/// extern crate nobs_vk as vk;
/// extern crate nobs_vkmem as vkmem;
///
/// use vkmem::Handle;
///
/// struct Ub {
///   a: u32,
///   b: u32,
///   c: u32,
/// }
///
/// # fn main() {
/// #  let lib = vk::VkLib::new();
/// #  let inst = vk::instance::new()
/// #    .validate(vk::DEBUG_REPORT_ERROR_BIT_EXT | vk::DEBUG_REPORT_WARNING_BIT_EXT)
/// #    .application("awesome app", 0)
/// #    .add_extension(vk::KHR_SURFACE_EXTENSION_NAME)
/// #    .add_extension(vk::KHR_XLIB_SURFACE_EXTENSION_NAME)
/// #    .create(lib)
/// #    .unwrap();
/// #  let (pdevice, device) = vk::device::PhysicalDevice::enumerate_all(inst.handle)
/// #    .remove(0)
/// #    .into_device()
/// #    .add_queue(vk::device::QueueProperties {
/// #      present: false,
/// #      graphics: true,
/// #      compute: true,
/// #      transfer: true,
/// #    }).create()
/// #    .unwrap();
///
/// // crate the allocator and trash
/// // the inflight parameter of Trash::new() controlls the delay of resource deletion
/// let mut allocator = vkmem::Allocator::new(pdevice.handle, device.handle);
/// let mut trash = vkmem::Trash::new(allocator.clone(), 2);
///
/// // create some resources
/// let mut buf = vk::NULL_HANDLE;
/// let mut img = vk::NULL_HANDLE;
/// // vkmem::Buffer::new(&mut buf)...
/// # vkmem::Buffer::new(&mut buf)
/// #  .size(std::mem::size_of::<Ub>() as vk::DeviceSize)
/// #  .usage(vk::BUFFER_USAGE_TRANSFER_DST_BIT | vk::BUFFER_USAGE_UNIFORM_BUFFER_BIT)
/// #  .devicelocal(false)
/// #  .new_image(&mut img)
/// #  .image_type(vk::IMAGE_TYPE_2D)
/// #  .size(123, 123, 1)
/// #  .usage(vk::IMAGE_USAGE_SAMPLED_BIT)
/// #  .devicelocal(true)
/// #  .bind(&mut allocator, vkmem::BindType::Scatter)
/// #  .unwrap();
///
/// // do some thing ...
///
/// // we are done using img, so mark it in trash
/// trash.push_image(img);
///
/// // this will do nothing, because we used 2 as inflight parameter when constructing trash
/// trash.clean();
///
/// // suppose we finished our draw call (and synced with the cpu)
/// // now clean will delete img
/// trash.clean();
/// # }
/// ```
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
