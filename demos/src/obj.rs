extern crate nobs_assets as assets;
extern crate nobs_imgui as imgui;
#[macro_use]
extern crate nobs_vkmath as vkm;
#[macro_use]
extern crate nobs_vulkanism as vk;

mod camera;

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
  pub struct UbModel {
    pub model: vkm::Mat4f,
  }

  #[repr(C)]
  #[derive(Clone, Copy)]
  pub struct UbCamera {
    pub view: vkm::Mat4f,
    pub proj: vkm::Mat4f,
  }

  #[repr(C)]
  #[allow(dead_code)]
  pub struct Vertex {
    pub pos: vkm::Vec3f,
    pub norm: vkm::Vec3f,
    pub uv: vkm::Vec2f,
  }

  use vk::builder::Buildable;
  use vk::pipes as p;

  pub struct Pipeline {
    pipe: p::Pipeline,
    pool: p::DescriptorPool,
  }

  impl Pipeline {
    pub fn new(device: vk::Device, pass: vk::RenderPass) -> Self {
      let pipe = new(device, pass, 0)
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
        .create()
        .unwrap();

      let pool = p::DescriptorPool::new(device, p::DescriptorPool::new_capacity().add(&pipe.dsets[0], 1));

      Self { pipe, pool }
    }

    pub fn new_dset(&mut self) -> vk::cmd::commands::BindDset {
      let ds = self.pool.new_dset(&self.pipe.dsets[0]).unwrap();
      vk::cmd::commands::BindDset::new(vk::PIPELINE_BIND_POINT_GRAPHICS, self.pipe.layout, 0, ds)
    }

    pub fn bind(&self) -> vk::cmd::commands::BindPipeline {
      vk::cmd::commands::BindPipeline::graphics(self.pipe.handle)
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

pub fn resize_window(
  pdevice: &vk::device::PhysicalDevice,
  device: &vk::device::Device,
  window: &vk::wnd::Window,
  alloc: &mut vk::mem::Allocator,
  mut sc: Option<vk::wnd::Swapchain>,
  pass: Option<vk::pass::Renderpass>,
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

pub fn main() {
  let (_inst, pdevice, device, mut events_loop, window) = setup_vulkan_window();

  let mut alloc = vk::mem::Allocator::new(pdevice.handle, device.handle);
  let cmds = vk::cmd::CmdPool::new(device.handle, device.queues[0].family).unwrap();

  let (mut sc, mut rp, mut fb) = resize_window(&pdevice, &device, &window, &mut alloc, None, None, None);
  let mut mem = vk::mem::Mem::new(alloc, 1);

  let mut camera = camera::Camera::new(mem.clone());
  camera.transform.view = vkm::Mat4::look_at(vec3!(0.0, 0.0, -10.0), vec3!(0.0, 0.0, 0.0), vec3!(0.0, 1.0, 0.0));
  camera.resize(sc.extent);

  use assets::Asset;
  let mut assets = std::collections::HashMap::new();
  let mut up = assets::Update::new(mem.clone());
  let shapes = &assets::model::wavefront::Asset::get(&"assets/bunny.obj".to_string(), &mut assets, &mut up).shapes;

  let draw = DrawManaged::new(
    [(shapes[0].vertices, 0), (shapes[0].normals, 0), (shapes[0].uvs, 0)].iter().into(),
    DrawIndexed::with_indices(shapes[0].indices, shapes[0].count)
      .index_type(vk::INDEX_TYPE_UINT32)
      .into(),
  );

  let mut pipe = obj::Pipeline::new(device.handle, rp.pass);
  let bind_ds = pipe.new_dset();

  let mut ub_model = vk::NULL_HANDLE;
  vk::mem::Buffer::new(&mut ub_model)
    .uniform_buffer(std::mem::size_of::<obj::UbModel>() as vk::DeviceSize)
    .bind(&mut mem.alloc, vk::mem::BindType::Block)
    .unwrap();

  // update uniform buffers
  {
    let mut stage = up.get_staging(std::mem::size_of::<obj::UbModel>() as vk::DeviceSize);
    let map = stage.map().unwrap();
    map.host_to_device(&obj::UbModel {
      model: vkm::Mat4::scale(vec3!(1.0, 1.0, -1.0)),
    });
    up.push_buffer((
      stage.copy_into_buffer(ub_model, 0),
      Some(BufferBarrier::new(ub_model).to(vk::ACCESS_UNIFORM_READ_BIT)),
    ));
  }

  obj::dset::write(device.handle, bind_ds.dset)
    .ub_model(vk::DescriptorBufferInfo::build().buffer(ub_model).into())
    .ub_camera(vk::DescriptorBufferInfo::build().buffer(camera.ub).into())
    .update();

  let mut close = false;
  let mut resize = false;

  use vk::cmd::commands::*;
  let mut frame = vk::cmd::RRBatch::new(device.handle, 1).unwrap();
  loop {
    events_loop.poll_events(|event| {
      camera.handle_events(&event);
      match event {
        winit::Event::WindowEvent {
          event: winit::WindowEvent::CloseRequested,
          ..
        } => close = true,
        winit::Event::WindowEvent {
          event: winit::WindowEvent::Resized(_),
          ..
        } => {
          resize = true;
        }
        winit::Event::WindowEvent {
          event: winit::WindowEvent::HiDpiFactorChanged(_),
          ..
        } => {
          resize = true;
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
      }
    });

    if resize {
      vk_uncheck!(vk::DeviceWaitIdle(device.handle));
      let (nsc, nrp, nfb) = resize_window(&pdevice, &device, &window, &mut mem.alloc, Some(sc), Some(rp), Some(fb));
      sc = nsc;
      rp = nrp;
      fb = nfb;

      camera.resize(sc.extent);
      resize = false;

      println!("{:?}", sc.extent);
    }

    camera.update(&mut up);

    frame.next().unwrap();
    let next = sc.next_image();

    let cs = cmds
      .begin_stream()
      .unwrap()
      .push_mut(&mut up)
      .push(&ImageBarrier::to_color_attachment(fb.images[0]))
      .push(&fb.begin())
      .push(&Viewport::with_extent(sc.extent))
      .push(&Scissor::with_extent(sc.extent))
      .push(&pipe.bind())
      .push(&bind_ds)
      .push(&draw)
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

  println!("{}", mem.alloc.print_stats());
}
