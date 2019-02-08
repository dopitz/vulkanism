mod device;
mod physical_device;

/// Errors that can happen during device creation
#[derive(Debug)]
pub enum Error {
  /// Indicates, that one or more [QueueProperties](struct.QueueProperties.html) that have been requested with [add_queue](struct.Builder.html#method.add_queue)
  /// is not supported on the physical device
  UnsuppordetQueue,
  /// Indicates, that the vulkan 'CreateDevice' command failed. The vulkan error code is the stored in the enum's interal value.
  DeviceCreateFailed(crate::Error),
}

pub use device::Builder;
pub use device::Device;
pub use device::Queue;
pub use device::QueueProperties;
pub use physical_device::PhysicalDevice;
