extern crate cgmath as cgm;
extern crate nobs_imgui as imgui;
extern crate nobs_vulkanism as vk;

use vk::builder::Buildable;
use vk::winit;

pub fn setup_vulkan_window() -> (
  vk::instance::Instance,
  vk::device::PhysicalDevice,
  vk::device::Device,
  winit::EventsLoop,
  vk::wnd::Window,
) {
  let lib = vk::VkLib::new();
  let inst = vk::instance::new()
    .validate(vk::DEBUG_REPORT_ERROR_BIT_EXT | vk::DEBUG_REPORT_WARNING_BIT_EXT)
    .application("awesome app", 0)
    .add_extension(vk::KHR_SURFACE_EXTENSION_NAME)
    .add_extension(vk::KHR_XLIB_SURFACE_EXTENSION_NAME)
    .create(lib)
    .unwrap();

  let events_loop = winit::EventsLoop::new();

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

  (inst, pdevice, device, events_loop, window)
}

pub fn resize(
  pdevice: &vk::device::PhysicalDevice,
  device: &vk::device::Device,
  window: &vk::wnd::Window,
  alloc: &mut vk::mem::Allocator,
  sc: Option<vk::wnd::Swapchain>,
  rp: Option<vk::fb::Renderpass>,
  fbs: Option<Vec<vk::fb::Framebuffer>>,
) -> (vk::wnd::Swapchain, vk::fb::Renderpass, Vec<vk::fb::Framebuffer>) {
  if sc.is_some() {
    sc.unwrap();
  }
  if rp.is_some() {
    rp.unwrap();
  }
  if fbs.is_some() {
    fbs.unwrap();
  }

  let sc = vk::wnd::Swapchain::build(pdevice.handle, device.handle, window.surface).create();

  let depth_format = vk::fb::select_depth_format(pdevice.handle, vk::fb::DEPTH_FORMATS).unwrap();

  let pass = vk::fb::Renderpass::build(device.handle)
    .attachment(0, vk::AttachmentDescription::build().format(vk::FORMAT_B8G8R8A8_UNORM))
    .attachment(1, vk::AttachmentDescription::build().format(depth_format))
    .subpass(
      0,
      vk::SubpassDescription::build()
        .bindpoint(vk::PIPELINE_BIND_POINT_GRAPHICS)
        .color(0)
        .depth(1),
    )
    .dependency(vk::SubpassDependency::build().external(0))
    .create()
    .unwrap();

  println!("{:?}", sc.extent);

  let fbs = vec![
    vk::fb::Framebuffer::build_from_pass(&pass, alloc).extent(sc.extent).create(),
    vk::fb::Framebuffer::build_from_pass(&pass, alloc).extent(sc.extent).create(),
    vk::fb::Framebuffer::build_from_pass(&pass, alloc).extent(sc.extent).create(),
  ];

  (sc, pass, fbs)
}

pub fn main() {
  let (_inst, pdevice, device, mut events_loop, window) = setup_vulkan_window();

  let mut alloc = vk::mem::Allocator::new(pdevice.handle, device.handle);
  let cmds = vk::cmd::Pool::new(device.handle, device.queues[0].family).unwrap();

  let (mut sc, mut rp, mut fbs) = resize(&pdevice, &device, &window, &mut alloc, None, None, None);

  let mut gui = imgui::ImGui::new(device.handle, device.queues[0].handle, cmds.clone(), rp.pass, 0, alloc.clone());

  let mut text = imgui::text::Text::new(&gui, "aoueaoeu");

  let mut resizeevent = false;
  let mut close = false;
  let mut x = 'x';

  use vk::cmd::commands::*;
  let mut frame = vk::cmd::Frame::new(device.handle, fbs.len()).unwrap();

  loop {
    events_loop.poll_events(|event| match event {
      winit::Event::WindowEvent {
        event: winit::WindowEvent::CloseRequested,
        ..
      } => close = true,
      winit::Event::WindowEvent {
        event: winit::WindowEvent::ReceivedCharacter(c),
        ..
      } => x = c,
      winit::Event::WindowEvent {
        event: winit::WindowEvent::Resized(size),
        ..
      } => {
        println!("RESIZE       {:?}", size);
        resizeevent = true;
      }
      _ => (),
    });

    if resizeevent {
      let (nsc, nrp, nfbs) = resize(&pdevice, &device, &window, &mut alloc, Some(sc), Some(rp), Some(fbs));
      sc = nsc;
      rp = nrp;
      fbs = nfbs;

      gui.resize(sc.extent);

      resizeevent = false;
    }

    let i = frame.next().unwrap();
    let next = sc.next_image();
    let fb = &fbs[i];

    let cs = cmds
      .begin_stream()
      .unwrap()
      .push(&ImageBarrier::to_color_attachment(fb.images[0]))
      .push(&fb.begin())
      .push(&Viewport::with_extent(sc.extent))
      .push(&Scissor::with_extent(sc.extent));

    let cs = gui
      .begin(cs)
      .push(&mut text)
      .end()
      .unwrap()
      .push(&fb.end())
      .push(&sc.blit(next.index, fb.images[0]));

    let (_, wait) = frame
      .wait_for(next.signal, vk::PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT)
      .push(cs)
      .submit(device.queues[0].handle);

    sc.present(device.queues[0].handle, next.index, &[wait.unwrap()]);

    if close {
      break;
    }
  }

  frame.sync().unwrap();

  println!("{}", alloc.print_stats());
}
