extern crate nobs_imgui as imgui;
extern crate nobs_vulkanism as vk;
extern crate nobs_vkmath as vkm;

use vk::builder::Buildable;
use vk::cmd::stream::*;
use vk::mem::Handle;
use vk::winit;

struct Context {
  quit: bool,
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
      imgs.push(Handle::Image(*i));
    }
    alloc.destroy_many(&imgs);
  }

  let sc = vk::wnd::Swapchain::build(pdevice.handle, device.handle, window.surface).create();

  let depth_format = vk::pass::Framebuffer::select_depth_format(pdevice.handle, vk::pass::Framebuffer::enumerate_depth_formats()).unwrap();

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
  let cmds = vk::cmd::CmdPool::new(device.handle, device.queues[0].family).unwrap();

  let (mut sc, mut rp, mut fb) = resize(&pdevice, &device, &window, &mut alloc, None, None, None);
  let mem = vk::mem::Mem::new(alloc.clone(), 2);

  let mut gui = Gui::new(&device, &window, fb.images[0], mem.clone());

  let mut context = Context { quit: false };

  let mut resizeevent = false;

  use vk::cmd::commands::*;
  let mut batch = vk::cmd::RRBatch::new(device.handle, 1).unwrap();

  loop {
    events_loop.poll_events(|event| {
      match event {
        winit::Event::WindowEvent {
          event: winit::WindowEvent::CloseRequested,
          ..
        } => context.quit = true,
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
      gui.handle_events(&event);
    });

    if resizeevent {
      batch.sync().unwrap();
      vk::DeviceWaitIdle(device.handle);

      let (nsc, nrp, nfb) = resize(&pdevice, &device, &window, &mut alloc, Some(sc), Some(rp), Some(fb));
      sc = nsc;
      rp = nrp;
      fb = nfb;

      gui.resize(sc.extent, fb.images[0]);
      resizeevent = false;
    }

    mem.trash.clean();
    vk::wnd::PresentFrame::new(cmds.clone(), &mut sc, &mut batch)
      .push(&ImageBarrier::to_color_attachment(fb.images[0]))
      .push(&fb.begin())
      .push(&fb.end())
      .push_mut(&mut gui.render(&mut context))
      .present(device.queues[0].handle, fb.images[0]);

    if context.quit {
      break;
    }
  }

  batch.sync().unwrap();
}

use imgui::style::simple as gui;

mod commands {
  use crate::gui::shell::*;
  use crate::gui::*;
  use crate::Context;

  pub mod quit {
    use super::*;

    pub struct Cmd {
      name: String,
    }

    impl Command<ThisStyle, super::Context> for Cmd {
      fn get_name(&self) -> &str {
        &self.name
      }
      fn get_args<'a>(&'a self) -> Vec<&'a arg::Parsable> {
        vec![]
      }

      fn run(&self, _args: Vec<String>, _term: &Terminal, context: &mut super::Context) {
        context.quit = true;
      }
    }

    impl Cmd {
      pub fn new() -> Self {
        Self { name: "quit".to_owned() }
      }
    }
  }

  pub mod toggle {
    use super::*;

    pub struct Cmd {
      name: String,
      toggle: arg::Bool,
    }

    impl Command<ThisStyle, super::Context> for Cmd {
      fn get_name(&self) -> &str {
        &self.name
      }
      fn get_args<'a>(&'a self) -> Vec<&'a arg::Parsable> {
        vec![&self.toggle]
      }

      fn run(&self, args: Vec<String>, term: &Terminal, _context: &mut super::Context) {
        let on = self.toggle.convert(&args[1]);
        term.println(&format!("{:?}", on));
      }
    }

    impl Cmd {
      pub fn new() -> Self {
        Self {
          name: "toggle".to_owned(),
          toggle: arg::Bool::new(),
        }
      }
    }
  }
}

struct Gui {
  gui: gui::Gui,

  shell: gui::shell::Shell<Context>,

  //wnd: gui::window::Window<gui::window::ColumnLayout>,
  //text: gui::components::TextEditMultiline,
  //text2: gui::components::TextBox,

  focus: gui::select::SelectId,
}

impl Gui {
  pub fn new(device: &vk::device::Device, wnd: &vk::wnd::Window, target: vk::Image, mem: vk::mem::Mem) -> Self {
    use gui::*;
    let mut gui = gui::Gui::new(device, wnd, target, mem);
    gui.style.load_styles(gui::get_default_styles());
    gui.style.set_dpi(1.6);

    let mut shell = gui::shell::Shell::new(&gui);
    shell.add_command(Box::new(commands::toggle::Cmd::new()));
    shell.add_command(Box::new(commands::quit::Cmd::new()));

    //let mut wnd = gui::window::Window::new(&gui, gui::window::ColumnLayout::default());
    //wnd.caption("awwwww yeees").position(200, 20).size(500, 720).focus(true).draw_caption(false);
    //wnd
    //  .caption("awwwww yeees")
    //  .position(200, 20)
    //  .size(500, 720)
    //  .focus(true)
    //  .padding(vec2!(10));

    //let mut text = gui::components::TextBox::new(&gui);
    //text.text("aoeu\naoeu\naoeu");
    //text.cursor(Some(vec2!(1, 0)));

    //let mut text2 = gui::components::TextBox::new(&gui);
    //text2.text("aoeu\naoeu\naoeu\naoeu");
    //text2.typeset(text2.get_typeset());
    Self {
      gui,
      shell,
      //wnd,
      //text,
      //text2,
      focus: imgui::select::SelectId::invalid(),
    }
  }

  pub fn handle_events(&mut self, e: &vk::winit::Event) {
    self.gui.handle_events(e);
  }

  pub fn resize(&mut self, extent: vk::Extent2D, image: vk::Image) {
    self.gui.resize(extent, image);
    self.shell.term.size(extent.width / 7 * 3, extent.height / 4 * 3);
  }

  pub fn render<'a>(&'a mut self, context: &'a mut Context) -> RenderGui<'a> {
    RenderGui { context, gui: self }
  }
}

struct RenderGui<'a> {
  context: &'a mut Context,
  gui: &'a mut Gui,
}

impl<'a> StreamPushMut for RenderGui<'a> {
  fn enqueue_mut(&mut self, cs: CmdBuffer) -> CmdBuffer {
    let gui = &mut self.gui;

    let mut scr = gui.gui.begin();
    let mut layout = gui::window::FloatLayout::from(scr.get_rect());

    //self.wnd.draw(&mut scr, &mut layout, &mut self.focus);
    //if let Some(e) = self.text.draw(&mut scr, &mut self.wnd, &mut self.focus) {
    //  self.wnd.focus(true);
    //};

    //gui::Spacer::new(vec2!(10)).draw(&mut scr, &mut self.wnd, &mut self.focus);

    //if let Some(e) = self.text2.draw(&mut scr, &mut self.wnd, &mut self.focus) {
    //  self.wnd.focus(true);
    //};

    gui.shell.update(&mut scr, &mut layout, &mut gui.focus, self.context);

    cs.push_mut(&mut scr)
  }
}
