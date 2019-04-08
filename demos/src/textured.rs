extern crate cgmath as cgm;
extern crate nobs_vulkanism as vk;

use cgm::*;
use vk::builder::Buildable;
use vk::winit;

mod tex {
  vk::pipes::pipeline! {
    stage = {
      ty = "vert",
      glsl = "src/textured.vert",
    }

    stage = {
      ty = "frag",
      glsl = "src/textured.frag",
    }
  }

  #[derive(Clone, Copy)]
  pub struct UbTransform {
    pub model: cgm::Matrix4<f32>,
    pub view: cgm::Matrix4<f32>,
    pub proj: cgm::Matrix4<f32>,
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
) -> (vk::wnd::Swapchain, vk::fb::Renderpass, Vec<vk::fb::Framebuffer>) {
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

  let fbs = vec![
    vk::fb::Framebuffer::build_from_pass(&pass, alloc).extent(sc.extent).create(),
    vk::fb::Framebuffer::build_from_pass(&pass, alloc).extent(sc.extent).create(),
    vk::fb::Framebuffer::build_from_pass(&pass, alloc).extent(sc.extent).create(),
  ];

  (sc, pass, fbs)
}

pub fn update_mvp(device: &vk::device::Device, cmds: &vk::cmd::Pool, stage: &mut vk::mem::Staging, ub: vk::Buffer, mvp: &tex::UbTransform) {
  let mut map = stage
    .range(0, std::mem::size_of::<tex::UbTransform>() as vk::DeviceSize)
    .map()
    .unwrap();
  let svb = map.as_slice_mut::<tex::UbTransform>();

  svb[0] = *mvp;

  let cs = cmds.begin_stream().unwrap().push(&stage.copy_into_buffer(ub, 0));

  let mut batch = vk::cmd::AutoBatch::new(device.handle).unwrap();
  batch.push(cs).submit(device.queues[0].handle).0.sync().unwrap();
}

pub fn main() {
  let (inst, pdevice, device, mut events_loop, window) = setup_vulkan_window();

  let mut alloc = vk::mem::Allocator::new(pdevice.handle, device.handle);
  let cmds = vk::cmd::Pool::new(device.handle, device.queues[0].family).unwrap();

  let (mut sc, rp, fbs) = setup_rendertargets(&pdevice, &device, &window, &mut alloc);

  let pipe = tex::new(device.handle, rp.pass)
    .vertex_input(
      vk::PipelineVertexInputStateCreateInfo::build()
        .push_binding(
          vk::VertexInputBindingDescription::build()
            .binding(0)
            .stride(4 * std::mem::size_of::<f32>() as u32)
            .binding,
        )
        .push_attribute(vk::VertexInputAttributeDescription::build().binding(0).location(0).attribute),
    )
    .dynamic(
      vk::PipelineDynamicStateCreateInfo::build()
        .push_state(vk::DYNAMIC_STATE_VIEWPORT)
        .push_state(vk::DYNAMIC_STATE_SCISSOR),
    )
    .blend(vk::PipelineColorBlendStateCreateInfo::build().push_attachment(vk::PipelineColorBlendAttachmentState::build()))
    .create()
    .unwrap();

  let mut vb = vk::NULL_HANDLE;
  let mut ub = vk::NULL_HANDLE;
  vk::mem::Buffer::new(&mut vb)
    .vertex_buffer(12 * std::mem::size_of::<f32>() as vk::DeviceSize)
    .new_buffer(&mut ub)
    .uniform_buffer(std::mem::size_of::<tex::UbTransform>() as vk::DeviceSize)
    .bind(&mut alloc, vk::mem::BindType::Block)
    .unwrap();

  let mut stage = vk::mem::Staging::new(&mut alloc, std::mem::size_of::<tex::UbTransform>() as vk::DeviceSize).unwrap();
  {
    let mut map = stage.range(0, 12 * std::mem::size_of::<f32>() as vk::DeviceSize).map().unwrap();
    let svb = map.as_slice_mut::<f32>();
    svb[0] = 0.0;
    svb[1] = 1.0;
    svb[2] = 0.0;
    svb[3] = 1.0;

    svb[4] = -1.0;
    svb[5] = -1.0;
    svb[6] = 0.0;
    svb[7] = 1.0;

    svb[8] = 1.0;
    svb[9] = -1.0;
    svb[10] = 0.0;
    svb[11] = 1.0;

    let cs = cmds.begin_stream().unwrap().push(&stage.copy_into_buffer(vb, 0));

    let mut batch = vk::cmd::AutoBatch::new(device.handle).unwrap();
    batch.push(cs).submit(device.queues[0].handle).0.sync().unwrap();
  }

  let mut descriptors = vk::pipes::DescriptorPool::with_capacity(device.handle, &tex::SIZES, tex::NUM_SETS).unwrap();
  let ds = descriptors.new_dset(pipe.dsets[&0].layout, &pipe.dsets[&0].sizes).unwrap();

  tex::dset::write(device.handle, ds).ub_transform(|b| b.buffer(ub)).update();

  let t = std::time::SystemTime::now();
  let mut n = 0;

  let mut close = false;
  let mut x = 'x';

  use vk::cmd::commands::*;
  let draw = Draw::default().push(vb, 0).vertices().vertex_count(3);
  let mut frame = vk::cmd::Frame::new(device.handle, fbs.len()).unwrap();

  let mut z = -10.0;

  let mut mvp = tex::UbTransform {
    model: cgm::One::one(),
    view: cgm::Matrix4::look_at(
      cgm::Point3::new(0.0, 0.0, z),
      cgm::Point3::new(0.0, 0.0, 0.0),
      cgm::Vector3::new(0.0, 1.0, 0.0),
    ),
    proj: cgm::Matrix4::from_nonuniform_scale(1.0, -1.0, 1.0) * cgm::perspective(cgm::Deg(45.0), 1.0, 1.0, 100.0),
  };

  loop {
    events_loop.poll_events(|event| match event {
      winit::Event::WindowEvent {
        event: winit::WindowEvent::CloseRequested,
        ..
      } => close = true,
      winit::Event::WindowEvent {
        event: winit::WindowEvent::Resized(size),
        ..
      } => {
        println!("{:?}", size);
        update_mvp(&device, &cmds, &mut stage, ub, &mvp)
      }
      winit::Event::WindowEvent {
        event: winit::WindowEvent::ReceivedCharacter(c),
        ..
      } => x = c,
      winit::Event::DeviceEvent {
        event: winit::DeviceEvent::Key(key),
        ..
      } => {
        if let Some(k) = key.virtual_keycode {
          match k {
            winit::VirtualKeyCode::Comma => {
              z += 0.2;
              mvp.view = cgm::Matrix4::look_at(
                cgm::Point3::new(0.0, 0.0, z),
                cgm::Point3::new(0.0, 0.0, 0.0),
                cgm::Vector3::new(0.0, 1.0, 0.0),
              );
              update_mvp(&device, &cmds, &mut stage, ub, &mvp);
            }
            winit::VirtualKeyCode::O => {
              z -= 0.2;
              mvp.view = cgm::Matrix4::look_at(
                cgm::Point3::new(0.0, 0.0, z),
                cgm::Point3::new(0.0, 0.0, 0.0),
                cgm::Vector3::new(0.0, 1.0, 0.0),
              );
              update_mvp(&device, &cmds, &mut stage, ub, &mvp);
            }
            _ => (),
          }
        }

        //println!("{:?}", key);
      }
      winit::Event::DeviceEvent {
        event: winit::DeviceEvent::MouseWheel {
          delta: winit::MouseScrollDelta::LineDelta(lp, la),
        },
        ..
      } => {
        println!("{:?}", (lp, la));
      }
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
      .push(&BindDset::new(vk::PIPELINE_BIND_POINT_GRAPHICS, pipe.layout, 0, ds))
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

  println!("{}", alloc.print_stats());

  let t = t.elapsed().unwrap();
  let t = t.as_secs() as f32 + t.subsec_millis() as f32 / 1000.0;
  println!("{}, {}   {}", n, t, n as f32 / t);
}
