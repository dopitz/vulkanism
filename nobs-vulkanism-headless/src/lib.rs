//! Compilation of vulkanism modules to render offstreen / compute
//!
//! This library is a curation of the all vulkanism modules. Including:
//!  - [nobs-vk](https://docs.rs/nobs-vk)
//!  - [nobs-vkcmd](https://docs.rs/nobs-vkcmd)
//!  - [nobs-vkmem](https://docs.rs/nobs-vkmem)
//!  - [nobs-vkpipes](https://docs.rs/nobs-vkpipes)
//!
//! Rearranges module namespaces, so that we only have to use a single `external crate nobs_vulkanism` instruction.
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
extern crate nobs_vk;
extern crate nobs_vkcmd;
extern crate nobs_vkmem;
extern crate nobs_vkpipes;

pub use nobs_vk::*;
pub mod cmd {
  pub use nobs_vkcmd::*;
}
pub mod mem {
  pub use nobs_vkmem::*;
}
pub mod pipes {
  pub use nobs_vkpipes::*;
}
