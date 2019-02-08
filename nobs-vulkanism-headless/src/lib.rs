//! vulkan for offscreen rendering and compute
//!
//! This library compiles the nobs-vk base crates into a single depencency and introduces two more moduls:
//!  - [cmd](mod.cmd.html) - Handles command buffers and syrchronization
//!  - [fm](mod.fb.html) - Handles renderpass and framebuffer management
//!
//! Rearranges module namespaces, so that we only have to use a single `external crate nobs_vulkanism` instruction instead of
//! the all three depencencies (nobs-vk, nobs-vkmem, nobs-vkpipes). Inlines the nobs-vk Symbols into thes crates root namespace.
//! for nobs-vkmem and nobs-vkpipes the modules `mem` and `pipes` are created respectively.
//!
//! ## Example
//! ```rust
//! extern crate nobs_vulkanism_headless as vk;
//!
//! fn main() {
//!   // nobs-vk Symbols remain in vk::*
//!   let lib = vk::Core::new();
//!   let inst = vk::instance::new()
//!     .validate(vk::DEBUG_REPORT_ERROR_BIT_EXT | vk::DEBUG_REPORT_WARNING_BIT_EXT)
//!     .application("awesome app", 0)
//!     .create(lib)
//!     .unwrap();
//!
//!   let (pdevice, device) = vk::device::PhysicalDevice::enumerate_all(inst.handle)
//!     .remove(0)
//!     .into_device()
//!     .add_queue(vk::device::QueueProperties {
//!       present: false,
//!       graphics: true,
//!       compute: true,
//!       transfer: true,
//!     })
//!     .create()
//!     .unwrap();
//!
//!   // Symbols of dependent moduls are put in their own namespace within vk::
//!   // e.g.:
//!   let mut allocator = vk::mem::Allocator::new(pdevice.handle, device.handle);
//!   //...
//! }
//! ```
#[macro_use]
extern crate nobs_vk as vk;
extern crate nobs_vkmem;
extern crate nobs_vkpipes;

pub use vk::*;
pub mod mem {
  pub use nobs_vkmem::*;
}
pub mod pipes {
  pub use nobs_vkpipes::*;
}

pub mod cmd;
pub mod fb;
