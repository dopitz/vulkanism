use std::ffi::CStr;
use std::ffi::CString;
use std::os::raw::*;
use std::ptr;

use crate as vk;

/// Wrapps the core library with an instance
///
/// Manages a vulkan instance with an optional debug callback for validation layers
///
/// The Instance will take ownership of the [VkLib](../struct.VkLib.html):
///  - there is no need to ever have multiple vulkan instances in the same process.
///  - the Instance and vulkan core library should go out of scope simultanously.
///
/// Create and conveniently configure an Instance with the [instance builder](fn.new.html)
pub struct Instance {
  #[allow(dead_code)]
  vklib: std::boxed::Box<vk::VkLib>,
  debug_callback: vk::DebugReportCallbackEXT,
  /// The actual vulkan instance handle
  pub handle: vk::Instance,
}

impl Drop for Instance {
  /// Cleans up the Instance
  ///
  /// If the instance was created with validation layers cleans up the debug callback.
  /// Destroys the instance handle.
  ///
  /// After the instance is dropped, all vulkan commands will panic
  fn drop(&mut self) {
    if self.debug_callback != vk::NULL_HANDLE {
      let name = std::ffi::CString::new("vkDestroyDebugReportCallbackEXT").unwrap();
      let ptr = vk::GetInstanceProcAddr(self.handle, name.as_ptr());
      if !(ptr as *const c_void).is_null() {
        let destroy_callback: vk::PFN_vkDestroyDebugReportCallbackEXT = unsafe { std::mem::transmute(ptr) };
        destroy_callback(self.handle, self.debug_callback, ptr::null());
      };
    }
    vk::DestroyInstance(self.handle, ptr::null());
  }
}

extern "system" fn debug_callback_fn(
  _flags: vk::DebugReportFlagsEXT,
  _objecttype: vk::DebugReportObjectTypeEXT,
  _object: u64,
  _location: usize,
  _messagecode: i32,
  _playerprefix: *mut c_char,
  p_message: *mut c_char,
  _puserdata: *mut c_void,
) -> vk::Bool32 {
  unsafe {
    let cstr = CStr::from_ptr(p_message).to_str().unwrap();
    println!("validation layer: {}", cstr);
  }
  vk::FALSE
}

/// Builder for an [Instance](struct.Instance.html)
///
/// Configures validation layers, application, engine name and used extensions for the new instance.
///
/// Creating an Instance will take ownership of the specified [VkLib](../struct.VkLib.html) object.
/// However it is still necessary to create it before the Builder, so that we can look up supported layers and extensions, while configuring.
///
/// ## Example
/// ```
/// #[macro_use]
/// extern crate nobs_vk as vk;
///
/// # fn main() {
/// let vk_lib = vk::VkLib::new();
/// let inst = vk::instance::new()
///   .validate(vk::DEBUG_REPORT_ERROR_BIT_EXT | vk::DEBUG_REPORT_WARNING_BIT_EXT)
///   .application("awesome app", make_version!(1, 0, 0))
///   .engine("very fast with lots of PS", make_version!(1, 33, 7))
///   // add extensions like this
///   //.add_extension(vk::KHR_SURFACE_EXTENSION_NAME)
///   //.add_extension(vk::KHR_XLIB_SURFACE_EXTENSION_NAME)
///   .create(vk_lib)
///   .expect("instance creation failed");
/// # }
/// ```
pub struct Builder {
  application_name: CString,
  application_verson: u32,
  engine_name: CString,
  engine_version: u32,

  layer_names: Vec<CString>,
  extension_names: Vec<CString>,

  validation_flags: vk::DebugReportFlagsEXT,
}

impl Default for Builder {
  /// Initializes builder with no layers, no extensions and no validation
  fn default() -> Builder {
    Builder {
      application_name: CString::new("vulkan application").unwrap(),
      application_verson: make_version!(1, 0, 0),
      engine_name: CString::new("no engine").unwrap(),
      engine_version: make_version!(1, 0, 0),

      layer_names: Default::default(),
      extension_names: Default::default(),

      validation_flags: 0,
    }
  }
}

/// Create a new [instance builder](struct.Builder.html) with default configuration
pub fn new() -> Builder {
  Builder::default()
}

impl Builder {
  /// Enables or disables validation layers.
  ///
  /// # Arguments
  /// * `flags` - chooses between
  ///   - `0` - validation disabled
  ///   - other - automatically adds the `VK_LAYER_LUNARG_standard_validation` layer and configure the debug callback
  pub fn validate(&mut self, flags: vk::DebugReportFlagsEXT) -> &mut Self {
    self.validation_flags = flags;
    self
  }

  /// Set the application name and version
  ///
  /// # Arguments
  /// * `name` - application name
  /// * `version` - application version, as created with [make_version](../macro.make_version.html)
  pub fn application(&mut self, name: &str, version: u32) -> &mut Self {
    self.application_name = CString::new(name).expect("invalid application name");
    self.application_verson = version;
    self
  }

  /// Set the engine name and version
  ///
  /// # Arguments
  /// * `name` - engine name
  /// * `version` - engine version, as created with [make_version](../macro.make_version.html)
  pub fn engine(&mut self, name: &str, version: u32) -> &mut Self {
    self.engine_name = CString::new(name).expect("invalid engine name");
    self.engine_version = version;
    self
  }

  /// Retrieve a list of all supported layers on this system
  ///
  /// # Returns
  /// The names of all supported layers in this vulkan instance.
  ///
  /// The `vk::EnumerateInstanceLayerProperties` command can fail, in this case an empty list is returned
  pub fn get_supported_layers() -> Vec<String> {
    let mut layer_count = 0u32;
    if let Err(_) = vk_check!(vk::EnumerateInstanceLayerProperties(&mut layer_count, ptr::null_mut())) {
      return Vec::new();
    }

    let mut layers = Vec::with_capacity(layer_count as usize);
    if let Err(_) = vk_check!(vk::EnumerateInstanceLayerProperties(&mut layer_count, layers.as_mut_ptr())) {
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

  /// Check if the specified name is a supported layer
  pub fn is_layer_supported(name: &str) -> bool {
    Self::get_supported_layers().iter().any(|l| l == name)
  }

  /// Adds a layer, if it is supported
  pub fn add_layer(&mut self, name: &str) -> &mut Self {
    if !self.layer_names.iter().any(|n| n.to_str().unwrap() == name) && Self::is_layer_supported(name) {
      self.layer_names.push(CString::new(name).unwrap());
    }
    self
  }

  /// Adds layers, if they are supported
  pub fn add_layers(&mut self, names: &[&str]) -> &mut Self {
    names.iter().fold(self, |b, n| b.add_layer(n))
  }

  /// Retrieve a list of all supported extensions on this system
  ///
  /// # Returns
  /// The names of all supported extensions in this vulkan instance.
  ///
  /// The `vk::EnumerateInstanceExtensionProperties` command can fail, in this case an empty list is returned
  pub fn get_supported_extensions(layer: Option<&str>) -> Vec<String> {
    let layer_cstr = layer.map(|n| CString::new(n).unwrap());
    let layer_ptr = match layer_cstr {
      Some(cstr) => cstr.as_ptr(),
      None => ptr::null(),
    };

    let mut ext_count = 0u32;
    if let Err(_) = vk_check!(vk::EnumerateInstanceExtensionProperties(
      ptr::null(),
      &mut ext_count,
      ptr::null_mut()
    )) {
      return Vec::new();
    }

    let mut extensions = Vec::with_capacity(ext_count as usize);
    if let Err(_) = vk_check!(vk::EnumerateInstanceExtensionProperties(
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

  /// Check if the specified name is a supported extension
  pub fn is_extension_supported(layer: Option<&str>, name: &str) -> bool {
    Self::get_supported_extensions(layer).iter().any(|l| l == name)
  }

  /// Adds an extension, if it is supported
  pub fn add_extension(&mut self, name: &str) -> &mut Self {
    if !self.extension_names.iter().any(|n| n.to_str().unwrap() == name) && Self::is_extension_supported(None, name) {
      self.extension_names.push(CString::new(name).unwrap());
    }
    self
  }

  /// Adds extensions, if they are supported
  pub fn add_extensions(&mut self, names: &[&str]) -> &mut Self {
    names.iter().fold(self, |b, n| b.add_extension(n))
  }

  /// Create the instance from the current configuration
  ///
  /// # Returns
  /// Instance creation can only fail, if the `vk::CreateInstance` call is unsuccessfull.
  /// In this case the [vk::Error](../enum.Error.html) is returned.
  pub fn create(&mut self, vklib: std::boxed::Box<vk::VkLib>) -> Result<Instance, vk::Error> {
    let app_info = vk::ApplicationInfo {
      sType: vk::STRUCTURE_TYPE_APPLICATION_INFO,
      pNext: ptr::null(),
      pApplicationName: self.application_name.as_ptr(),
      applicationVersion: self.application_verson,
      pEngineName: self.engine_name.as_ptr(),
      engineVersion: self.engine_version,
      apiVersion: vklib.get_feature(),
    };

    if self.validation_flags != 0 {
      self.add_layer("VK_LAYER_LUNARG_standard_validation");
      self.add_extension(vk::EXT_DEBUG_REPORT_EXTENSION_NAME);
    }

    let layers_ptr: Vec<*const c_char> = self.layer_names.iter().map(|l| l.as_ptr()).collect();
    let extensions_ptr: Vec<*const c_char> = self.extension_names.iter().map(|e| e.as_ptr()).collect();

    let create_info = vk::InstanceCreateInfo {
      sType: vk::STRUCTURE_TYPE_INSTANCE_CREATE_INFO,
      pNext: ptr::null(),
      flags: 0,
      pApplicationInfo: &app_info,
      enabledLayerCount: layers_ptr.len() as u32,
      ppEnabledLayerNames: layers_ptr.as_ptr(),
      enabledExtensionCount: extensions_ptr.len() as u32,
      ppEnabledExtensionNames: extensions_ptr.as_ptr(),
    };

    let mut handle = vk::NULL_HANDLE;
    vk_check!(vk::CreateInstance(&create_info, ptr::null(), &mut handle))?;

    let mut debug_callback = vk::NULL_HANDLE;
    if self.validation_flags != 0 {
      let name = std::ffi::CString::new("vkCreateDebugReportCallbackEXT").unwrap();
      let ptr = vk::GetInstanceProcAddr(handle, name.as_ptr());
      if !(ptr as *const c_void).is_null() {
        let create_callback: vk::PFN_vkCreateDebugReportCallbackEXT = unsafe { std::mem::transmute(ptr) };
        let callback_info = vk::DebugReportCallbackCreateInfoEXT {
          sType: vk::STRUCTURE_TYPE_DEBUG_REPORT_CALLBACK_CREATE_INFO_EXT,
          pNext: ptr::null(),
          flags: self.validation_flags,
          pfnCallback: debug_callback_fn,
          pUserData: ptr::null_mut(),
        };

        vk_check!(create_callback(handle, &callback_info, ptr::null(), &mut debug_callback))?;
      };
    }

    Ok(Instance {
      vklib,
      debug_callback,
      handle,
    })
  }
}
