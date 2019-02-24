use std::ffi::CString;
use std::os::raw::*;
use std::ptr;

use crate as vk;
use crate::device::PhysicalDevice;
use crate::device::Error;

/// Flags to describe a queue's capabilities
#[derive(Debug, Clone, Copy)]
pub struct QueueProperties {
  pub present: bool,
  pub graphics: bool,
  pub compute: bool,
  pub transfer: bool,
}

/// Wrapper for a queue and it's index on a device
#[derive(Debug, Clone, Copy)]
pub struct Queue {
  pub properties: QueueProperties,
  pub handle: vk::Queue,
  pub family: u32,
  pub index: u32,
}

/// Wrapper for a successfully created logical vulkan device
///
/// Tracks the lifetime of the vulkan device handle and queues
///
/// Create and conveniently configure a Device with the [device builder](struct.Builder.html)
pub struct Device {
  /// The actuol vulkan device handle
  pub handle: vk::Device,
  /// Queues of the device
  pub queues: Vec<Queue>,
}

impl Drop for Device {
  /// Cleans up device handle and queues
  fn drop(&mut self) {
    vk::DestroyDevice(self.handle, ptr::null());
  }
}

/// Implements the builder pattern for [Device](struct.Device.html)
///
/// Configures validation layers, used extensions, used queues and surface
///
/// # Example
///  - Create an instance (see [here](../instance.Builder.html))
///  - Enumerates all physical devices and picks the first one.
///  - Creates a builder from the physical device and cofigures it
///  - Finally [create](struct.Builder.html#method.create) will create the device and set up the queues
/// ```rust
/// #[macro_use]
/// extern crate nobs_vk as vk;
/// # fn main() {
/// // create instance ...
/// # let vk_lib = vk::Core::new();
/// # let inst = vk::instance::new()
/// #   .validate(vk::DEBUG_REPORT_ERROR_BIT_EXT | vk::DEBUG_REPORT_WARNING_BIT_EXT)
/// #   .application("awesome app", make_version!(1, 0, 0))
/// #   .engine("very fast with lots of PS", make_version!(1, 33, 7))
/// #   .add_extension(vk::KHR_SURFACE_EXTENSION_NAME)
/// #   .add_extension(vk::KHR_XLIB_SURFACE_EXTENSION_NAME)
/// #   .create(vk_lib)
/// #   .unwrap();
/// let (pdevice, device) = vk::device::PhysicalDevice::enumerate_all(inst.handle)
///   .remove(0)
///   .into_device()
///   .add_queue(vk::device::QueueProperties {
///     present: false,
///     graphics: true,
///     compute: true,
///     transfer: true,
///   }).create()
///   .expect("device creation failed");
/// # }
/// ```
pub struct Builder {
  physical_device: PhysicalDevice,
  layer_names: Vec<CString>,
  extension_names: Vec<CString>,
  queues: Vec<QueueProperties>,
  surface: vk::SurfaceKHR,
}

impl Builder {
  /// Creates a device builder from the physical device.
  ///
  /// The physical device is returned as the first item of the tuple in [create](struct.Builder.html#methad.create) again.
  pub fn from_physical_device(physical_device: PhysicalDevice) -> Builder {
    Builder {
      physical_device,
      layer_names: Default::default(),
      extension_names: Default::default(),
      queues: Default::default(),
      surface: vk::NULL_HANDLE,
    }
  }

  /// Adds a layer, if it is supported
  pub fn add_layer(&mut self, name: &str) -> &mut Self {
    if !self.layer_names.iter().any(|n| n.to_str().unwrap() == name) && self.physical_device.is_layer_supported(name) {
      self.layer_names.push(CString::new(name).unwrap());
    }
    self
  }

  /// Adds layers, if they are supported
  pub fn add_layers(&mut self, names: &[&str]) -> &mut Self {
    names.iter().fold(self, |b, n| b.add_layer(n))
  }

  /// Adds an extension, if it is supported
  pub fn add_extension(&mut self, name: &str) -> &mut Self {
    if !self.extension_names.iter().any(|n| n.to_str().unwrap() == name) && self.physical_device.is_extension_supported(name) {
      self.extension_names.push(CString::new(name).unwrap());
    }
    self
  }

  /// Adds extensions, if they are supported
  pub fn add_extensions(&mut self, names: &[&str]) -> &mut Self {
    names.iter().fold(self, |b, n| b.add_extension(n))
  }

  /// Adds a queue with the requested properties
  pub fn add_queue(&mut self, properties: QueueProperties) -> &mut Self {
    self.queues.push(properties);
    self
  }

  /// Specify a surface for devices that can present to a window
  pub fn surface(&mut self, surface: vk::SurfaceKHR) -> &mut Self {
    self.surface = surface;
    self
  }

  /// Creates the device
  ///
  /// # Returns
  /// A tuple with the [PhysicalDevice](struct.PhysicalDevice.html) from which the builder was created and the new [Device](struct.Device.html).
  ///
  /// This function fails if
  ///  - A queue that was specified with [add_queue](struct.Builder.html#method.add_queue) is not supported
  ///  - The `vk::CreateDevice` command fails
  pub fn create(&mut self) -> Result<(PhysicalDevice, Device), Error> {
    struct Family {
      properties: QueueProperties,
      index: u32,
      max_count: u32,
      count: u32,
      priorities: Vec<f32>,
    }

    // Get all available queue families from vulkan
    let mut families: Vec<Family> = {
      // Get family properties from vulkan
      let mut family_count: u32 = 0;
      vk::GetPhysicalDeviceQueueFamilyProperties(self.physical_device.handle, &mut family_count, ptr::null_mut());

      let mut queues = Vec::with_capacity(family_count as usize);
      vk::GetPhysicalDeviceQueueFamilyProperties(self.physical_device.handle, &mut family_count, queues.as_mut_ptr());

      unsafe { queues.set_len(family_count as usize) };

      // Find out surface support for each queue family
      // Map them to Family, so that we can track how many queues of what family we need
      queues
        .iter()
        .enumerate()
        .map(|f| Family {
          properties: QueueProperties {
            present: match self.surface {
              vk::NULL_HANDLE => false,
              _ => {
                let mut present_support = vk::FALSE;
                vk::GetPhysicalDeviceSurfaceSupportKHR(self.physical_device.handle, f.0 as u32, self.surface, &mut present_support);
                present_support == vk::TRUE
              }
            },
            graphics: f.1.queueFlags & vk::QUEUE_GRAPHICS_BIT != 0,
            compute: f.1.queueFlags & vk::QUEUE_COMPUTE_BIT != 0,
            transfer: f.1.queueFlags & vk::QUEUE_TRANSFER_BIT != 0,
          },
          index: f.0 as u32,
          max_count: f.1.queueCount,
          count: 0,
          priorities: Vec::new(),
        }).collect()
    };

    // Initialize the queues we will later retrieve from the device with invalid indices and handles
    let mut queues: Vec<Queue> = self
      .queues
      .iter()
      .map(|q| Queue {
        properties: *q,
        handle: vk::NULL_HANDLE,
        family: !0u32,
        index: !0u32,
      }).collect();

    // Find a matching queue family for every queue that was requested in the Builder
    for q in queues.iter_mut().enumerate() {
      for family in families.iter_mut() {
        let properties = q.1.properties;

        let good = family.count < family.max_count
          && properties.present == family.properties.present
          && properties.graphics == family.properties.graphics
          && properties.compute == family.properties.compute
          && properties.transfer == family.properties.transfer;

        if good {
          q.1.family = family.index;
          q.1.index = family.count;
          family.count += 1;
          family.priorities.push(1f32);
          break;
        }
      }
    }

    if queues.iter().any(|q| q.family == !0u32 || q.index == !0u32) {
      return Err(Error::UnsuppordetQueue);
    }

    let queue_infos: Vec<vk::DeviceQueueCreateInfo> = families
      .iter()
      .filter(|f| f.count > 0)
      .map(|f| vk::DeviceQueueCreateInfo {
        sType: vk::STRUCTURE_TYPE_DEVICE_QUEUE_CREATE_INFO,
        pNext: ptr::null(),
        flags: 0,
        queueFamilyIndex: f.index,
        queueCount: f.count,
        pQueuePriorities: f.priorities.as_ptr(),
      }).collect();

    let layers: Vec<*const c_char> = self.layer_names.iter().map(|l| l.as_ptr()).collect();
    let extensions: Vec<*const c_char> = self.extension_names.iter().map(|e| e.as_ptr()).collect();

    // Create the device
    let create_info = vk::DeviceCreateInfo {
      sType: vk::STRUCTURE_TYPE_DEVICE_CREATE_INFO,
      pNext: ptr::null(),
      flags: 0,
      queueCreateInfoCount: queue_infos.len() as u32,
      pQueueCreateInfos: queue_infos.as_ptr(),
      enabledLayerCount: layers.len() as u32,
      ppEnabledLayerNames: layers.as_ptr(),
      enabledExtensionCount: extensions.len() as u32,
      ppEnabledExtensionNames: extensions.as_ptr(),
      pEnabledFeatures: &self.physical_device.features,
    };

    let mut handle = vk::NULL_HANDLE;

    vk_check!(vk::CreateDevice(
      self.physical_device.handle,
      &create_info,
      ptr::null(),
      &mut handle,
    )).map_err(|e| Error::DeviceCreateFailed(e))?;

    // Retrieve queues
    queues
      .iter_mut()
      .for_each(|q| vk::GetDeviceQueue(handle, q.family, q.index, &mut q.handle));

    Ok((
      self.physical_device.clone(),
      Device {
        handle,
        queues,
      },
    ))
  }
}
