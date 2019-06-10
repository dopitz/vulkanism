extern crate nobs_imgui as imgui;
extern crate nobs_vulkanism as vk;
#[macro_use]
extern crate nobs_vkmath as vkm;

use vk::builder::Buildable;
use vk::pass::Pass;
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
  rp: Option<vk::pass::Renderpass>,
  fb: Option<vk::pass::Framebuffer>,
) -> (vk::wnd::Swapchain, vk::pass::Renderpass, vk::pass::Framebuffer) {
  if sc.is_some() {
    sc.unwrap();
  }
  if rp.is_some() {
    rp.unwrap();
  }
  if fb.is_some() {
    let fb = fb.unwrap();
    let mut imgs = Vec::new();
    for i in fb.images.iter() {
      imgs.push(*i);
    }
    alloc.destroy_many(&imgs);
  }

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

  let fb = vk::pass::Framebuffer::build_from_pass(&pass, alloc).extent(sc.extent).create();

  (sc, pass, fb)
}

pub fn main() {
  let (_inst, pdevice, device, mut events_loop, window) = setup_vulkan_window();

  let mut alloc = vk::mem::Allocator::new(pdevice.handle, device.handle);
  let cmds = vk::cmd::Pool::new(device.handle, device.queues[0].family).unwrap();

  let (mut sc, mut rp, mut fb) = resize(&pdevice, &device, &window, &mut alloc, None, None, None);
  let mem = vk::mem::Mem::new(alloc.clone(), 2);

  let mut gui = Gui::new(&device, cmds.clone(), sc.extent, fb.images[0], mem.clone());

  let mut resizeevent = false;
  let mut close = false;

  use vk::cmd::commands::*;
  let mut frame = vk::cmd::Frame::new(device.handle, 1).unwrap();

  loop {
    events_loop.poll_events(|event| {
      match event {
        winit::Event::WindowEvent {
          event: winit::WindowEvent::CloseRequested,
          ..
        } => close = true,
        winit::Event::WindowEvent {
          event: winit::WindowEvent::ReceivedCharacter(c),
          ..
        } => gui.input(c),
        winit::Event::WindowEvent {
          event: winit::WindowEvent::Resized(size),
          ..
        } => {
          println!("RESIZE       {:?}", size);
          println!("EVENT        {:?}", event);
          println!("DPI          {:?}", window.window.get_hidpi_factor());

          if fb.extent.width != size.width as u32 || fb.extent.height != size.height as u32 {
            resizeevent = true;
          }
        }
        winit::Event::WindowEvent {
          event: winit::WindowEvent::HiDpiFactorChanged(dpi),
          ..
        } => {
          println!("DPI       {:?}", dpi);
        }
        _ => (),
      }
      //println!("{:?}", inp.parse(event));
    });

    if resizeevent {
      println!("AOEUAOEUAOEUAOEUAOEU");

      frame.sync().unwrap();
      let (nsc, nrp, nfb) = resize(&pdevice, &device, &window, &mut alloc, Some(sc), Some(rp), Some(fb));
      sc = nsc;
      rp = nrp;
      fb = nfb;

      gui.gui.resize(sc.extent);
      resizeevent = false;
    }

    mem.trash.clean();

    let i = frame.next().unwrap();
    let next = sc.next_image();

    let cs = cmds
      .begin_stream()
      .unwrap()
      .push(&ImageBarrier::to_color_attachment(fb.images[0]))
      .push(&fb.begin())
      //.push(&gui)
      //.push(&gui.begin_window().push(text.text(&t)))
      .push(&fb.end())
      .push_fnmut(|cs| gui.render(cs))
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

  println!("{}", alloc.print_stats());
  frame.sync().unwrap();
}

struct Gui {
  gui: imgui::ImGui,
  //text: imgui::text::Text,
  text: imgui::textbox::TextBox,

  tt: String,
}

impl Gui {
  pub fn new(device: &vk::device::Device, cmds: vk::cmd::Pool, extent: vk::Extent2D, target: vk::Image, mem: vk::mem::Mem) -> Self {
    let gui = imgui::ImGui::new(device.handle, device.queues[0].handle, cmds.clone(), extent, target, mem);

    let mut text = imgui::textbox::TextBox::new(&gui);
    text.text("aoeu").rect(imgui::rect::Rect::new(vec2!(200, 200), vec2!(500, 200)));
    text.typeset(text.get_typeset().size(70).cursor(Some(vec2!(1, 0))));
    Self {
      gui,
      text,
      tt: Default::default(),
    }
  }

  pub fn input(&mut self, c: char) {
    self.text.text(&format!("{}{}", self.text.get_text(), c));
    self.tt.push(c);
  }

  pub fn render(&self, cs: vk::cmd::Stream) -> vk::cmd::Stream {
    cs.push(&self.gui)
      .push(&self.gui.begin_window())
      .push(&self.text)
      .push(&self.gui.end())
    //cs.push(&self.gui).push(&self.gui.begin_window()).push(self.text.text(&self.tt))
    //cs.push(&self.gui).push(&self.text)
  }
}
