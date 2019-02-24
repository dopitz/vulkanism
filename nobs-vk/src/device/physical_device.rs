use std::ffi::CStr;
use std::ffi::CString;
use std::ptr;

use crate as vk;
use crate::device::Builder;

/// Description and properties of a physical device
///
/// Used for [Device](../device/struct.Device.html) creation.
#[derive(Clone)]
pub struct PhysicalDevice {
  pub instance: vk::Instance,
  pub handle: vk::PhysicalDevice,
  pub supported_layers: Vec<String>,
  pub supported_extensions: Vec<String>,
  pub features: vk::PhysicalDeviceFeatures,
  pub properties: vk::PhysicalDeviceProperties,
}

impl PhysicalDevice {
  fn new(instance: vk::Instance, handle: vk::PhysicalDevice) -> PhysicalDevice {
    fn get_supported_layers(handle: vk::PhysicalDevice) -> Vec<String> {
      let mut layer_count = 0u32;
      if let Err(_) = vk_check!(vk::EnumerateDeviceLayerProperties(handle, &mut layer_count, ptr::null_mut())) {
        return Vec::new();
      }

      let mut layers = Vec::with_capacity(layer_count as usize);
      if let Err(_) = vk_check!(vk::EnumerateDeviceLayerProperties(handle, &mut layer_count, layers.as_mut_ptr())) {
        return Vec::new();
      }
      unsafe {
        layers.set_len(layer_count as usize);

        layers
          .iter_mut()
          .map(|l| CStr::from_ptr(l.layerName.as_mut_ptr()).to_str().unwrap().to_owned())
          .collect()
      }
    }

    fn get_supported_extensions(handle: vk::PhysicalDevice, layer: Option<&str>) -> Vec<String> {
      let layer_cstr = layer.map(|n| CString::new(n).unwrap());
      let layer_ptr = match layer_cstr {
        Some(cstr) => cstr.as_ptr(),
        None => ptr::null(),
      };

      let mut ext_count = 0u32;
      if let Err(_) = vk_check!(vk::EnumerateDeviceExtensionProperties(
        handle,
        ptr::null(),
        &mut ext_count,
        ptr::null_mut()
      )) {
        return Vec::new();
      }

      let mut extensions = Vec::with_capacity(ext_count as usize);
      if let Err(_) = vk_check!(vk::EnumerateDeviceExtensionProperties(
        handle,
        layer_ptr,
        &mut ext_count,
        extensions.as_mut_ptr()
      )) {
        return Vec::new();
      }
      unsafe {
        extensions.set_len(ext_count as usize);

        extensions
          .iter_mut()
          .map(|e| CStr::from_ptr(e.extensionName.as_mut_ptr()).to_str().unwrap().to_owned())
          .collect()
      }
    }

    let mut features = unsafe { std::mem::uninitialized() };
    vk::GetPhysicalDeviceFeatures(handle, &mut features);

    let mut properties = unsafe { std::mem::uninitialized() };
    vk::GetPhysicalDeviceProperties(handle, &mut properties);

    PhysicalDevice {
      instance,
      handle,
      supported_layers: get_supported_layers(handle),
      supported_extensions: get_supported_extensions(handle, None),
      features,
      properties,
    }
  }

  /// Lists all devices in the specified instance
  ///
  /// ## Example
  /// Prints out the device names of all physical devices.
  /// ```
  /// #[macro_use]
  /// extern crate nobs_vk as vk;
  /// # fn main() {
  /// // Create instance ...
  /// # let vk_lib = vk::VkLib::new();
  /// # let inst = vk::instance::new()
  /// #   .validate(vk::DEBUG_REPORT_ERROR_BIT_EXT | vk::DEBUG_REPORT_WARNING_BIT_EXT)
  /// #   .application("awesome app", make_version!(1, 0, 0))
  /// #   .engine("very fast with lots of PS", make_version!(1, 33, 7))
  /// #   .add_extension(vk::KHR_SURFACE_EXTENSION_NAME)
  /// #   .add_extension(vk::KHR_XLIB_SURFACE_EXTENSION_NAME)
  /// #   .create(vk_lib)
  /// #   .unwrap();
  /// let devices = vk::device::PhysicalDevice::enumerate_all(inst.handle);
  /// for d in devices.iter() {
  ///   println!("{}", unsafe { std::ffi::CStr::from_ptr(d.properties.deviceName.as_ptr()).to_str().unwrap() });
  /// }
  /// # }
  /// ```
  pub fn enumerate_all(inst: vk::Instance) -> Vec<PhysicalDevice> {
    let mut num_devices = 0u32;
    if let Err(_) = vk_check!(vk::EnumeratePhysicalDevices(inst, &mut num_devices, ptr::null_mut())) {
      return Vec::new();
    }

    let mut devices: Vec<vk::PhysicalDevice> = Vec::with_capacity(num_devices as usize);
    if let Err(_) = vk_check!(vk::EnumeratePhysicalDevices(inst, &mut num_devices, devices.as_mut_ptr())) {
      return Vec::new();
    }
    unsafe {
      devices.set_len(num_devices as usize);
      devices.iter().map(|d| Self::new(inst, *d)).collect()
    }
  }

  /// Consumes the Physical device and converts it into a [device::Builder](../device/struct.Bulider.html)
  ///
  /// See [here]() to filter devices from [enumerate_all](../device/struct.PhysicalDevice.html#method.enumerate_all) for asserting requirements.
  /// See [Device](../device/struct.Builder.html) for how to create a logical device.
  pub fn into_device(self) -> Builder {
    Builder::from_physical_device(self)
  }

  /// Check if the specified name is a supported layer
  pub fn is_layer_supported(&self, name: &str) -> bool {
    self.supported_layers.iter().any(|l| l == name)
  }

  /// Check if the specified name is a supported extension
  pub fn is_extension_supported(&self, name: &str) -> bool {
    self.supported_extensions.iter().any(|l| l == name)
  }
}
