//! vulkan for rendering to a window
//!
//! This library is an extension to nobs-vulkanism-headless that adds the module wnd
//!  - [wnd](mod.wnd.html) - Handles window and swapchain creation
//!
//! Inludes the symbols from nobs-vulkanism-headless into che crate's root namespace and defines the wnd module.
//!
//! ## Example
//! ```rust
//! extern crate nobs_vulkanism as vk;
//!
//! fn main() {
//!   // nobs-vk Symbols remain in vk::*
//!   let lib = vk::VkLib::new();
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
extern crate nobs_vulkanism_headless as vk;
pub extern crate winit;

pub mod wnd;

pub use vk::*;
