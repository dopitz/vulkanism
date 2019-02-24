//! Manage vulkan windows and swapchains
//!
//! [Winwow](struct.Window.html) handles the creation of a swapchain for a [winit window](../../winit/index.html)
//!
//! [Swapchain](swapchain/struct.Swapchain.html) wrapps a vulkan swapchain.

mod window;
pub mod swapchain;

pub use window::Window;
pub use swapchain::Swapchain;
