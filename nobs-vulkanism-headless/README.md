# nobs-vulkanism-headless
Compilation of vulkanism modules to render to a window

This library is a curation of the all vulkanism modules. Including:
 - [nobs-vk](https://docs.rs/nobs-vk)
 - [nobs-vkcmd](https://docs.rs/nobs-vkcmd)
 - [nobs-vkmem](https://docs.rs/nobs-vkmem)
 - [nobs-vkpipes](https://docs.rs/nobs-vkpipes)

Rearranges module namespaces, so that we only have to use a single `external crate nobs_vulkanism` instruction.

## Example
```rust
extern crate nobs_vulkanism as vk;

fn main() {
  // nobs-vk Symbols remain in vk::*
  let lib = vk::Core::new();
  let inst = vk::instance::new()
    .validate(vk::DEBUG_REPORT_ERROR_BIT_EXT | vk::DEBUG_REPORT_WARNING_BIT_EXT)
    .application("awesome app", 0)
    .add_extension(vk::KHR_SURFACE_EXTENSION_NAME)
    .add_extension(vk::KHR_XLIB_SURFACE_EXTENSION_NAME)
    .create(lib)
    .unwrap();

  let (pdevice, device) = vk::device::PhysicalDevice::enumerate_all(inst.handle)
    .remove(0)
    .into_device()
    .add_extension(vk::KHR_SWAPCHAIN_EXTENSION_NAME)
    .add_queue(vk::device::QueueProperties {
      present: false,
      graphics: true,
      compute: true,
      transfer: true,
    })
    .create()
    .unwrap();

  // Symbols of dependent moduls are put in their own namespace within vk::
  // e.g.:
  let mut allocator = vk::mem::Allocator::new(pdevice.handle, device.handle);
  //...
}
```
