extern crate nobs_vulkanism as vk;

use vk::builder::Buildable;
use vk::cmd::stream::*;
use vk::winit;

mod tri {
  vk::pipes::pipeline! {
    stage = {
      ty = "vert",
      glsl = "src/triangle.vert",
    }

    stage = {
      ty = "frag",
      glsl = "src/triangle.frag",
    }
  }
}

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

pub fn setup_rendertargets(
  pdevice: &vk::device::PhysicalDevice,
  device: &vk::device::Device,
  window: &vk::wnd::Window,
  alloc: &mut vk::mem::Allocator,
) -> (vk::wnd::Swapchain, vk::pass::Renderpass, Vec<vk::pass::Framebuffer>) {
  let sc = vk::wnd::Swapchain::build(pdevice.handle, device.handle, window.surface).create();

  let depth_format = vk::pass::select_depth_format(pdevice.handle, vk::pass::DEPTH_FORMATS).unwrap();

  let pass = vk::pass::Renderpass::build(device.handle)
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

  let fbs = vec![
    vk::pass::Framebuffer::build_from_pass(&pass, alloc).extent(sc.extent).create(),
    vk::pass::Framebuffer::build_from_pass(&pass, alloc).extent(sc.extent).create(),
    vk::pass::Framebuffer::build_from_pass(&pass, alloc).extent(sc.extent).create(),
  ];

  //fbs[0].set_clear(&[vk::fb::clear_colorf32([1.0,0.0,0.0,1.0]), vk::fb::clear_depth(0.0)]);
  //fbs[1].set_clear(&[vk::fb::clear_colorf32([0.0,1.0,0.0,1.0]), vk::fb::clear_depth(0.0)]);
  //fbs[2].set_clear(&[vk::fb::clear_colorf32([0.0,0.0,1.0,1.0]), vk::fb::clear_depth(0.0)]);

  (sc, pass, fbs)
}

pub fn main() {
  let (_inst, pdevice, device, mut events_loop, window) = setup_vulkan_window();

  let mut alloc = vk::mem::Allocator::new(pdevice.handle, device.handle);
  let cmds = vk::cmd::CmdPool::new(device.handle, device.queues[0].family).unwrap();

  let (mut sc, rp, fbs) = setup_rendertargets(&pdevice, &device, &window, &mut alloc);

  let pipe = tri::new(device.handle, rp.pass, 0)
    .dynamic(
      vk::PipelineDynamicStateCreateInfo::build()
        .push_state(vk::DYNAMIC_STATE_VIEWPORT)
        .push_state(vk::DYNAMIC_STATE_SCISSOR),
    )
    .blend(vk::PipelineColorBlendStateCreateInfo::build().push_attachment(vk::PipelineColorBlendAttachmentState::build()))
    .create()
    .unwrap();

  let t = std::time::SystemTime::now();
  let mut n = 0;

  let mut close = false;
  let mut x = 'x';

  use vk::cmd::commands::*;
  let draw = Draw::new(Default::default(), DrawVertices::with_vertices(3).into());
  let mut frame = vk::cmd::RRBatch::new(device.handle, fbs.len()).unwrap();

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
      _ => (),
    });

    let i = frame.next().unwrap();
    let next = sc.next_image();
    let fb = &fbs[i];

    let cs = cmds
      .begin_stream()
      .unwrap()
      .push(&ImageBarrier::to_color_attachment(fb.images[0]))
      .push(&fb.begin())
      .push(&Viewport::with_extent(sc.extent))
      .push(&Scissor::with_extent(sc.extent))
      .push(&BindPipeline::graphics(pipe.handle))
      .push(&draw)
      .push(&fb.end())
      .push(&sc.blit(next.index, fb.images[0]));

    let (_, wait) = frame
      .wait_for(next.signal, vk::PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT)
      .push(cs)
      .submit(device.queues[0].handle);

    sc.present(device.queues[0].handle, next.index, &[wait.unwrap()]);
    n += 1;

    if close {
      break;
    }
  }

  let t = t.elapsed().unwrap();
  let t = t.as_secs() as f32 + t.subsec_millis() as f32 / 1000.0;
  println!("{}, {}   {}", n, t, n as f32 / t);
}
