extern crate nobs_assets as assets;
extern crate nobs_imgui as imgui;
#[macro_use]
extern crate nobs_vkmath as vkm;
#[macro_use]
extern crate nobs_vulkanism as vk;

use vk::builder::Buildable;
use vk::cmd::stream::*;
use vk::winit;

mod obj {
  vk::pipes::pipeline! {
    stage = {
      ty = "vert",
      glsl = "src/obj.vert",
    }

    stage = {
      ty = "frag",
      glsl = "src/obj.frag",
    }
  }

  #[repr(C)]
  #[derive(Clone, Copy)]
  pub struct UbTransform {
    pub model: vkm::Mat4f,
    pub view: vkm::Mat4f,
    pub proj: vkm::Mat4f,
  }

  #[repr(C)]
  pub struct Vertex {
    pub pos: vkm::Vec3f,
    pub norm: vkm::Vec3f,
    pub uv: vkm::Vec2f,
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

pub fn resize_window(
  pdevice: &vk::device::PhysicalDevice,
  device: &vk::device::Device,
  window: &vk::wnd::Window,
  alloc: &mut vk::mem::Allocator,
  mut sc: Option<vk::wnd::Swapchain>,
  mut pass: Option<vk::pass::Renderpass>,
  mut fb: Option<vk::pass::Framebuffer>,
) -> (vk::wnd::Swapchain, vk::pass::Renderpass, vk::pass::Framebuffer) {
  if sc.is_some() {
    sc.take();
  }
  let sc = vk::wnd::Swapchain::build(pdevice.handle, device.handle, window.surface).create();

  let depth_format = vk::pass::Framebuffer::select_depth_format(pdevice.handle, vk::pass::Framebuffer::enumerate_depth_formats()).unwrap();

  let pass = match pass {
    Some(p) => p,
    None => vk::pass::Renderpass::build(device.handle)
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
      .unwrap(),
  };

  if fb.is_some() {
    fb.take();
  }
  let fb = vk::pass::Framebuffer::build_from_pass(&pass, alloc).extent(sc.extent).create();

  (sc, pass, fb)
}

pub fn update_mvp(
  device: &vk::device::Device,
  cmds: &vk::cmd::CmdPool,
  stage: &mut vk::mem::Staging,
  ub: vk::Buffer,
  mvp: &obj::UbTransform,
) {
  let mut map = stage
    .range(0, std::mem::size_of::<obj::UbTransform>() as vk::DeviceSize)
    .map()
    .unwrap();
  let svb = map.as_slice_mut::<obj::UbTransform>();

  svb[0] = *mvp;

  let cs = cmds.begin_stream().unwrap().push(&stage.copy_into_buffer(ub, 0));

  let mut batch = vk::cmd::AutoBatch::new(device.handle).unwrap();
  batch.push(cs).submit(device.queues[0].handle).0.sync().unwrap();
}

pub fn main() {
  let (_inst, pdevice, device, mut events_loop, window) = setup_vulkan_window();

  let mut alloc = vk::mem::Allocator::new(pdevice.handle, device.handle);
  let cmds = vk::cmd::CmdPool::new(device.handle, device.queues[0].family).unwrap();

  let (mut sc, mut rp, mut fb) = resize_window(&pdevice, &device, &window, &mut alloc, None, None, None);
  let mut mem = vk::mem::Mem::new(alloc, 1);

  let pipe = obj::new(device.handle, rp.pass, 0)
    .vertex_input(
      vk::PipelineVertexInputStateCreateInfo::build()
        .push_binding(
          vk::VertexInputBindingDescription::build()
            .binding(0)
            .stride(std::mem::size_of::<vkm::Vec3f>() as u32),
        )
        .push_attribute(
          vk::VertexInputAttributeDescription::build()
            .binding(0)
            .location(0)
            .format(vk::FORMAT_R32G32B32_SFLOAT),
        )
        .push_binding(
          vk::VertexInputBindingDescription::build()
            .binding(1)
            .stride(std::mem::size_of::<vkm::Vec3f>() as u32),
        )
        .push_attribute(
          vk::VertexInputAttributeDescription::build()
            .binding(1)
            .location(1)
            .format(vk::FORMAT_R32G32B32_SFLOAT),
        )
        .push_binding(
          vk::VertexInputBindingDescription::build()
            .binding(2)
            .stride(std::mem::size_of::<vkm::Vec2f>() as u32),
        )
        .push_attribute(
          vk::VertexInputAttributeDescription::build()
            .binding(2)
            .location(2)
            .format(vk::FORMAT_R32G32_SFLOAT),
        ),
    )
    .dynamic(
      vk::PipelineDynamicStateCreateInfo::build()
        .push_state(vk::DYNAMIC_STATE_VIEWPORT)
        .push_state(vk::DYNAMIC_STATE_SCISSOR),
    )
    .blend(vk::PipelineColorBlendStateCreateInfo::build().push_attachment(vk::PipelineColorBlendAttachmentState::build()))
    //.raster(vk::PipelineRasterizationStateCreateInfo::build().front_face(vk::FRONT_FACE_CLOCKWISE))
    .create()
    .unwrap();

  let mut assets: assets::Assets<assets::model::wavefront::AssetType> = assets::Assets::new(device.handle, mem.clone());
  let shapes = assets.get("assets/bunny.obj");

  let mut ub = vk::NULL_HANDLE;
  vk::mem::Buffer::new(&mut ub)
    .uniform_buffer(std::mem::size_of::<obj::UbTransform>() as vk::DeviceSize)
    .bind(&mut mem.alloc, vk::mem::BindType::Block)
    .unwrap();

  let mut stage = vk::mem::Staging::new(mem.clone(), 256 * 256 * 4).unwrap();

  use vk::pipes::DescriptorPool;
  let descriptors = DescriptorPool::new(device.handle, DescriptorPool::new_capacity().add(&pipe.dsets[0], 1));
  let ds = descriptors.new_dset(&pipe.dsets[0]).unwrap();

  obj::dset::write(device.handle, ds)
    .ub_transform(vk::DescriptorBufferInfo::build().buffer(ub).into())
    .update();

  let t = std::time::SystemTime::now();
  let mut n = 0;

  let mut close = false;
  let mut resize = false;

  use vk::cmd::commands::*;
  let mut frame = vk::cmd::RRBatch::new(device.handle, 1).unwrap();

  let draw = DrawManaged::new(
    [(shapes[0].vertices, 0), (shapes[0].normals, 0), (shapes[0].uvs, 0)].iter().into(),
    DrawIndexed::with_indices(shapes[0].indices, shapes[0].count)
      .index_type(vk::INDEX_TYPE_UINT32)
      .into(),
  );

  let mut mvp = obj::UbTransform {
    model: vkm::Mat4::rotation_y(std::f32::consts::PI), //
    view: vkm::Mat4::look_at(
      vkm::Vec3::new(0.0, 0.0, -10.0),
      vkm::Vec3::new(0.0, 0.0, 0.0),
      vkm::Vec3::new(0.0, 1.0, 0.0),
    ),
    proj: vkm::Mat4::perspective_lh(std::f32::consts::PI / 4.0, 1.0, 1.0, 100.0),
  };

  let v = vec4!(1.0, 1.0, 1.0, 0.0);
  println!("{:?}", v);
  let v = mvp.model * v;
  println!("{:?}", v);
  let v = mvp.view * v;
  println!("{:?}", v);
  let v = mvp.proj * v;
  println!("{:?}", v);

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
        resize = true;
      }
      winit::Event::WindowEvent {
        event: winit::WindowEvent::HiDpiFactorChanged(dpi),
        ..
      } => {
        resize = true;
      }
      winit::Event::DeviceEvent {
        event: winit::DeviceEvent::Key(key),
        ..
      } => {
        if let Some(k) = key.virtual_keycode {
          match k {
            winit::VirtualKeyCode::Comma => {
              mvp.view = mvp.view * vkm::Mat4::translate(vkm::Vec3::new(0.0, 0.0, -0.2));
              update_mvp(&device, &cmds, &mut stage, ub, &mvp);
            }
            winit::VirtualKeyCode::O => {
              mvp.view = mvp.view * vkm::Mat4::translate(vkm::Vec3::new(0.0, 0.0, 0.2));
              update_mvp(&device, &cmds, &mut stage, ub, &mvp);
            }
            winit::VirtualKeyCode::L => {
              mvp.view = mvp.view * vkm::Mat4::translate(vkm::Vec3::new(-0.2, 0.0, 0.0));
              update_mvp(&device, &cmds, &mut stage, ub, &mvp);
            }
            winit::VirtualKeyCode::H => {
              mvp.view = mvp.view * vkm::Mat4::translate(vkm::Vec3::new(0.2, 0.0, 0.0));
              update_mvp(&device, &cmds, &mut stage, ub, &mvp);
            }
            _ => (),
          }
        }

        println!("{:?}", key);
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

    if resize {
      vk_uncheck!(vk::DeviceWaitIdle(device.handle));
      let (nsc, nrp, nfb) = resize_window(&pdevice, &device, &window, &mut mem.alloc, Some(sc), Some(rp), Some(fb));
      sc = nsc;
      rp = nrp;
      fb = nfb;

      mvp.proj = vkm::Mat4::scale(vec3!(-1.0, -1.0, 1.0))
        * vkm::Mat4::perspective_lh(
          std::f32::consts::PI / 4.0,
          sc.extent.width as f32 / sc.extent.height as f32,
          1.0,
          100.0,
        );
      update_mvp(&device, &cmds, &mut stage, ub, &mvp);
      resize = false;

      println!("{:?}", sc.extent);
    }

    let i = frame.next().unwrap();
    let next = sc.next_image();

    let cs = cmds
      .begin_stream()
      .unwrap()
      .push_mut(&mut assets.up)
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

  frame.sync().unwrap();

  println!("{}", mem.alloc.print_stats());

  let t = t.elapsed().unwrap();
  let t = t.as_secs() as f32 + t.subsec_millis() as f32 / 1000.0;
  println!("{}, {}   {}", n, t, n as f32 / t);
}
