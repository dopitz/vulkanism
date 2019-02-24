#[macro_use]
extern crate nobs_vk;

use nobs_vk as vk;
use std::ffi::CStr;
use std::mem;
use std::ptr;

fn main() {
  let _vk_lib = vk::VkLib::new();

  // global vulkan version
  let mut inst_ver: u32 = 0;
  vk_uncheck!(vk::EnumerateInstanceVersion(&mut inst_ver));
  println!(
    "system api version:  {} {} {}",
    version_major!(inst_ver),
    version_minor!(inst_ver),
    version_patch!(inst_ver)
  );

  // create an instance
  let appinfo = vk::InstanceCreateInfo {
    sType: vk::STRUCTURE_TYPE_INSTANCE_CREATE_INFO,
    pNext: ptr::null(),
    flags: 0,
    pApplicationInfo: ptr::null(),
    enabledLayerCount: 0,
    ppEnabledLayerNames: ptr::null(),
    enabledExtensionCount: 0,
    ppEnabledExtensionNames: ptr::null(),
  };

  let mut inst = vk::NULL_HANDLE;
  vk_uncheck!(vk::CreateInstance(&appinfo, ptr::null(), &mut inst));

  // devices
  let mut num_devices: u32 = 0;
  match vk_check!(vk::EnumeratePhysicalDevices(inst, &mut num_devices, ptr::null_mut())) {
    Err(e) => println!("EnumeratePhysicalDevices returned with: {:?}", e),
    Ok(e) => println!("EnumeratePhysicalDevices returned with: {:?}", e),
  }
  println!("num devices:  {}", num_devices);

  if num_devices == 0 {
    panic!("no devices found");
  }

  let mut phys_devices: Vec<vk::PhysicalDevice> = Vec::new();
  phys_devices.resize(num_devices as usize, vk::NULL_HANDLE);
  vk_check!(vk::EnumeratePhysicalDevices(inst, &mut num_devices, phys_devices.as_mut_ptr())).unwrap();

  let mut props: vk::PhysicalDeviceProperties = unsafe { mem::uninitialized() };
  vk::GetPhysicalDeviceProperties(phys_devices[0], &mut props);

  println!(
    "instance api version:  {} {} {}",
    version_major!(props.apiVersion),
    version_minor!(props.apiVersion),
    version_patch!(props.apiVersion)
  );
  println!("driver version:        {}", props.driverVersion);
  println!("vendor id:             {}", props.vendorID);
  println!("device id:             {}", props.deviceID);
  println!("vendor:                {}", unsafe {
    CStr::from_ptr(&props.deviceName[0]).to_str().unwrap()
  });

  vk::DestroyInstance(inst, ptr::null());
}
