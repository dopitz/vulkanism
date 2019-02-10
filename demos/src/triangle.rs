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

  let mut im_color = vk::NULL_HANDLE;
  let mut im_depth = vk::NULL_HANDLE;
  vk::mem::Image::new(&mut im_color)
    .color_attachment(512, 512, vk::FORMAT_B8G8R8A8_SRGB)
    .new_image(&mut im_depth)
    .depth_attachment(512, 512, depth_format)
    .bind(&mut alloc, vk::mem::BindType::Block)
    .unwrap();

  // TODO aspect
  let view_color = vk::mem::ImageView::new(device.handle, im_color)
    .format(vk::FORMAT_B8G8R8A8_SRGB)
    .aspect(vk::IMAGE_ASPECT_COLOR_BIT)
    .create()
    .unwrap();
  let view_depth = vk::mem::ImageView::new(device.handle, im_depth)
    .format(depth_format)
    .aspect(vk::IMAGE_ASPECT_DEPTH_BIT)
    .create()
    .unwrap();

  let fb = vk::fb::new_framebuffer(device.handle, pass.pass)
    .extent(vk::Extent2D { width: 512, height: 512 })
    .target(im_depth, view_depth, vk::fb::clear_depth(0.0))
    .target(im_color, view_color, vk::fb::clear_colorf32([0.0, 0.0, 0.0, 0.0]))
    .create();

  events_loop.run_forever(|event| {
    println!("{:?}", event);

    match event {
      winit::Event::WindowEvent {
        event: winit::WindowEvent::CloseRequested,
        ..
      } => winit::ControlFlow::Break,
      _ => winit::ControlFlow::Continue,
    }
  });
}
