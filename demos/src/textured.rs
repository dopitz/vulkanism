extern crate nobs_imgui as imgui;
extern crate nobs_vkmath as vkm;
extern crate nobs_vulkanism as vk;

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
    pub model: vkm::Mat4f,
    pub view: vkm::Mat4f,
    pub proj: vkm::Mat4f,
  }

  #[repr(C)]
  pub struct Vertex {
    pub pos: vkm::Vec4f,
    pub tex: vkm::Vec2f,
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
  let (_inst, pdevice, device, mut events_loop, window) = setup_vulkan_window();

  let mut alloc = vk::mem::Allocator::new(pdevice.handle, device.handle);
  let cmds = vk::cmd::Pool::new(device.handle, device.queues[0].family).unwrap();

  let (mut sc, rp, fbs) = setup_rendertargets(&pdevice, &device, &window, &mut alloc);

  let pipe = tex::new(device.handle, rp.pass, 0)
    .vertex_input(
      vk::PipelineVertexInputStateCreateInfo::build()
        .push_binding(
          vk::VertexInputBindingDescription::build()
            .binding(0)
            .stride(std::mem::size_of::<tex::Vertex>() as u32)
            .binding,
        )
        .push_attribute(vk::VertexInputAttributeDescription::build().binding(0).location(0).attribute)
        .push_attribute(
          vk::VertexInputAttributeDescription::build()
            .binding(0)
            .location(1)
            .format(vk::FORMAT_R32G32_SFLOAT)
            .offset(4 * std::mem::size_of::<f32>() as u32)
            .attribute,
        ),
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
  let mut texture = vk::NULL_HANDLE;
  vk::mem::Buffer::new(&mut vb)
    .vertex_buffer(3 * std::mem::size_of::<tex::Vertex>() as vk::DeviceSize)
    .new_buffer(&mut ub)
    .uniform_buffer(std::mem::size_of::<tex::UbTransform>() as vk::DeviceSize)
    .new_image(&mut texture)
    .texture2d(256, 256, vk::FORMAT_R8G8B8A8_UNORM)
    .bind(&mut alloc, vk::mem::BindType::Block)
    .unwrap();

  let texview = vk::ImageViewCreateInfo::build()
    .texture2d(texture, vk::FORMAT_R8G8B8A8_UNORM)
    .create(device.handle)
    .unwrap();
  let sampler = vk::SamplerCreateInfo::build().create(device.handle).unwrap();

  let mut stage = vk::mem::Staging::new(&mut alloc, 256 * 256 * 4).unwrap();
  {
    let mut map = stage
      .range(0, 3 * std::mem::size_of::<tex::Vertex>() as vk::DeviceSize)
      .map()
      .unwrap();
    let svb = map.as_slice_mut::<tex::Vertex>();
    svb[0].pos = vkm::Vec4::new(0.0, 1.0, 0.0, 1.0);
    svb[0].tex = vkm::Vec2::new(1.0, 1.0);

    svb[1].pos = vkm::Vec4::new(-1.0, -1.0, 0.0, 1.0);
    svb[1].tex = vkm::Vec2::new(1.0, 1.0);

    svb[2].pos = vkm::Vec4::new(1.0, -1.0, 0.0, 1.0);
    svb[2].tex = vkm::Vec2::new(1.0, 1.0);

    let cs = cmds.begin_stream().unwrap().push(&stage.copy_into_buffer(vb, 0));

    let mut batch = vk::cmd::AutoBatch::new(device.handle).unwrap();
    batch.push(cs).submit(device.queues[0].handle).0.sync().unwrap();
  }

  {
    let mut map = stage
      .range(0, 256 * 256 * std::mem::size_of::<u32>() as vk::DeviceSize)
      .map()
      .unwrap();
    let data = map.as_slice_mut::<u32>();

    for d in data.iter_mut() {
      *d = 0xFF << 24 | 0xFF << 8 | 0xFF;
    }

    let cs = cmds.begin_stream().unwrap().push(
      &stage.copy_into_image(
        texture,
        vk::BufferImageCopy::build()
          .image_extent(vk::Extent3D::build().set(256, 256, 1).extent)
          .subresource(vk::ImageSubresourceLayers::build().aspect(vk::IMAGE_ASPECT_COLOR_BIT).layers),
      ),
    );

    let mut batch = vk::cmd::AutoBatch::new(device.handle).unwrap();
    batch.push(cs).submit(device.queues[0].handle).0.sync().unwrap();
  }

  use vk::pipes::DescriptorPool;
  let descriptors = DescriptorPool::new(device.handle, DescriptorPool::new_capacity().add(&pipe.dsets[0], 1));
  let ds = descriptors.new_dset(&pipe.dsets[0]).unwrap();

  tex::dset::write(device.handle, ds)
    .ub_transform(|b| b.buffer(ub))
    .tex_sampler(|s| s.set(vk::IMAGE_LAYOUT_SHADER_READ_ONLY_OPTIMAL, texview, sampler))
    .update();

  let t = std::time::SystemTime::now();
  let mut n = 0;

  let mut close = false;
  let mut x = 'x';

  use vk::cmd::commands::*;
  let draw = Draw::default().push(vb, 0).vertices().vertex_count(3);
  let mut frame = vk::cmd::Frame::new(device.handle, fbs.len()).unwrap();

  let mut mvp = tex::UbTransform {
    model: vkm::Mat4::identity(),
    view: vkm::Mat4::look_at(
      vkm::Vec3::new(0.0, 0.0, -10.0),
      vkm::Vec3::new(0.0, 0.0, 0.0),
      vkm::Vec3::new(0.0, 1.0, 0.0),
    ),
    proj: vkm::Mat4::scale(vkm::Vec3::new(1.0, -1.0, 1.0)) * vkm::Mat4::perspective_lh(std::f32::consts::PI / 4.0, 1.0, 1.0, 100.0),
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
              mvp.view = mvp.view * vkm::Mat4::translate(vkm::Vec3::new(0.0, 0.0, -0.2));
              update_mvp(&device, &cmds, &mut stage, ub, &mvp);
            }
            winit::VirtualKeyCode::O => {
              mvp.view = mvp.view * vkm::Mat4::translate(vkm::Vec3::new(0.0, 0.0, 0.2));
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

  frame.sync().unwrap();

  vk::DestroyImageView(device.handle, texview, std::ptr::null());
  vk::DestroySampler(device.handle, sampler, std::ptr::null());

  println!("{}", alloc.print_stats());

  let t = t.elapsed().unwrap();
  let t = t.as_secs() as f32 + t.subsec_millis() as f32 / 1000.0;
  println!("{}, {}   {}", n, t, n as f32 / t);
}
