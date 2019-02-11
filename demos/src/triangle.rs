extern crate nobs_vulkanism as vk;

use vk::winit;

pub fn main() {
  let lib = vk::Core::new();
  let inst = vk::instance::new()
    .validate(vk::DEBUG_REPORT_ERROR_BIT_EXT | vk::DEBUG_REPORT_WARNING_BIT_EXT)
    .application("awesome app", 0)
    .add_extension(vk::KHR_SURFACE_EXTENSION_NAME)
    .add_extension(vk::KHR_XLIB_SURFACE_EXTENSION_NAME)
    .create(lib)
    .unwrap();

  let mut events_loop = winit::EventsLoop::new();

  let window = {
    let window = winit::WindowBuilder::new()
      .with_title("A fantastic window!")
      .build(&events_loop)
      .unwrap();

    vk::wnd::Window::new(inst.handle, window).unwrap()
  };

  let (pdevice, device) = vk::device::PhysicalDevice::enumerate_all(inst.handle)
    .remove(0)
    .into_device()
    .add_extension(vk::KHR_SWAPCHAIN_EXTENSION_NAME)
    .surface(window.surface)
    .add_queue(vk::device::QueueProperties {
      present: true,
      graphics: true,
      compute: true,
      transfer: true,
    })
    .create()
    .unwrap();

  let sc = vk::wnd::Swapchain::build(inst.handle, pdevice.handle, device.handle, window.surface).new();
  let mut alloc = vk::mem::Allocator::new(pdevice.handle, device.handle);

  let depth_format = vk::fb::select_depth_format(pdevice.handle, vk::fb::DEPTH_FORMATS).unwrap();

  let pass = vk::fb::new_pass(device.handle)
    .attachment(0, vk::fb::new_attachment(depth_format))
    .attachment(1, vk::fb::new_attachment(vk::FORMAT_B8G8R8A8_SRGB))
    .subpass(0, vk::fb::new_subpass(vk::PIPELINE_BIND_POINT_GRAPHICS).depth(0).color(1).clone())
    .dependency(vk::fb::new_dependency().external(0).clone())
    .create()
    .unwrap();

  let fb = vk::fb::new_framebuffer_from_pass(&pass, &mut alloc)
    .extent(vk::Extent2D { width: 512, height: 512 })
    .create();


  println!("{:?}", sc.images);

  println!("{:?}", fb.images);
  println!("{:?}", fb.views);

  events_loop.run_forever(|event| match event {
    winit::Event::WindowEvent {
      event: winit::WindowEvent::CloseRequested,
      ..
    } => winit::ControlFlow::Break,
    winit::Event::WindowEvent {
      event: winit::WindowEvent::ReceivedCharacter(c),
      ..
    } => {
      println!("{}", c);
      winit::ControlFlow::Continue
    }
    _ => winit::ControlFlow::Continue,
  });
}
