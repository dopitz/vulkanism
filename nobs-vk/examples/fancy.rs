#[macro_use]
extern crate nobs_vk;

use nobs_vk as vk;
use std::ffi::CStr;

fn main() {
  let lib = vk::Core::new();
  let inst = vk::instance::new()
    .validate(vk::DEBUG_REPORT_ERROR_BIT_EXT | vk::DEBUG_REPORT_WARNING_BIT_EXT)
    .application("awesome app", 0)
    .add_extension(vk::KHR_SURFACE_EXTENSION_NAME)
    .add_extension(vk::KHR_XLIB_SURFACE_EXTENSION_NAME)
    .create(lib)
    .unwrap();

  for pd in vk::device::PhysicalDevice::enumerate_all(inst.handle) {
    println!(
      "instance api version:  {} {} {}",
      version_major!(pd.properties.apiVersion),
      version_minor!(pd.properties.apiVersion),
      version_patch!(pd.properties.apiVersion)
    );
    println!("driver version:        {}", pd.properties.driverVersion);
    println!("vendor id:             {}", pd.properties.vendorID);
    println!("device id:             {}", pd.properties.deviceID);
    println!("vendor:                {}", unsafe {
      CStr::from_ptr(&pd.properties.deviceName[0]).to_str().unwrap()
    });
    
    println!("layers:                {:?}", pd.supported_layers);
    println!("extensions:            {:?}", pd.supported_extensions);
  }

  let (_pdevice, _device) = vk::device::PhysicalDevice::enumerate_all(inst.handle)
    .remove(0)
    .into_device()
    .add_queue(vk::device::QueueProperties {
      present: false,
      graphics: true,
      compute: true,
      transfer: true,
    }).create()
    .unwrap();
}
