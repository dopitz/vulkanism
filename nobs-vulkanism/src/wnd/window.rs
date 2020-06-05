use vk;
use winit;

#[derive(Debug)]
pub enum Error {
  NotSupported,
  SurfaceCreate(vk::Error),
}

/// Wrapper around a window plus a vulkan surface for drawing
pub struct Window {
  inst: vk::Instance,
  pub window: winit::window::Window,
  pub surface: vk::SurfaceKHR,
}

impl Window {
  /// Creates the window from a [winit window](../../winit/index.html) and a swapchain
  pub fn new(inst: vk::Instance, window: winit::window::Window) -> Result<Self, Error> {
    let surface = Self::create_surface(inst, &window)?;
    Ok(Self { inst, window, surface })
  }

  #[cfg(all(unix, not(target_os = "android"), not(target_os = "macos")))]
  fn create_surface(inst: vk::Instance, window: &winit::window::Window) -> Result<vk::SurfaceKHR, Error> {
    use winit::platform::unix::WindowExtUnix;
    if let Some(dpy) = window.xlib_display() {
      let window = window.xlib_window().unwrap();

      let info = vk::XlibSurfaceCreateInfoKHR {
        sType: vk::STRUCTURE_TYPE_XLIB_SURFACE_CREATE_INFO_KHR,
        pNext: std::ptr::null(),
        flags: 0,
        dpy,
        window,
      };

      let mut handle = vk::NULL_HANDLE;
      vk_check!(vk::CreateXlibSurfaceKHR(inst, &info, std::ptr::null(), &mut handle)).map_err(|e| Error::SurfaceCreate(e))?;
      return Ok(handle);
    }

    if let Some(display) = window.wayland_display() {
      let surface = window.wayland_surface().unwrap();

      let info = vk::WaylandSurfaceCreateInfoKHR {
        sType: vk::STRUCTURE_TYPE_WAYLAND_SURFACE_CREATE_INFO_KHR,
        pNext: std::ptr::null(),
        flags: 0,
        display,
        surface,
      };

      let mut handle = vk::NULL_HANDLE;
      vk_check!(vk::CreateWaylandSurfaceKHR(inst, &info, std::ptr::null(), &mut handle)).map_err(|e| Error::SurfaceCreate(e))?;
      return Ok(handle);

    }

    Err(Error::NotSupported)
  }

  #[cfg(target_os = "windows")]
  fn create_surface(inst: vk::Instance, window: &winit::window::Window) -> Result<vk::SurfaceKHR, vk::Error> {
    let ext = vk::InstanceExtensions::new(inst);
    let hwnd = window.get_hwnd();

    let info = vk::Win32SurfaceCreateInfoKHR {
      sType: vk::STRUCTURE_TYPE_XLIB_SURFACE_CREATE_INFO_KHR,
      pNext: std::ptr::null(),
      flags: 0,
      hinstance: std::ptr::null(),
      hwnd,
    };

    let mut handle = vk::NULL_HANDLE;
    vk_check!(vk::CreateWin32SurfaceKHR(inst, &info, std::ptr::null(), &mut handle))?;
    Ok(handle)
  }

  // TODO: more os
}

impl Drop for Window {
  fn drop(&mut self) {
    vk::DestroySurfaceKHR(self.inst, self.surface, std::ptr::null());
  }
}

