use crate::pipelines::Pipelines;
use crate::select::Select;
use crate::style::Style;
use crate::window::Screen;
use font::*;
use std::sync::Arc;
use std::sync::Mutex;
use vk::builder::Buildable;
use vk::mem::Handle;
use vk::pass::DrawPass;

struct Pass {
  rp: vk::pass::Renderpass,
  fb: Mutex<vk::pass::Framebuffer>,
  draw: Mutex<DrawPass>,
}

struct ImGuiImpl<S: Style> {
  device: vk::Device,
  mem: vk::mem::Mem,

  draw: Pass,
  ub_viewport: Mutex<vk::Buffer>,

  pipes: Mutex<Pipelines>,

  scr: Mutex<Option<Screen<S>>>,

  font: Arc<Font>,
}

impl<S: Style> Drop for ImGuiImpl<S> {
  fn drop(&mut self) {
    self.mem.trash.push_buffer(*self.ub_viewport.lock().unwrap());
  }
}

/// Main gui handle, controlls the renderpass, framebuffer object and event handling
///
/// The gui is clonable and implemented thread safe, hazards are handled internally.
///
/// This struct will be instantiated once. It creates a renderpass and framebuffer from the specidied vulkan image.
/// The gui also caches the vulkan pipelines and descriptor pools that are needed for rendering gui compontents.
///
/// Rendering a gui is done with associated funcions declared by gui components.
/// These funcions require a [Window](window/struct.Window.html) that handles positioning and resizing with [Layout](window/trait.Layout.html).
#[derive(Clone)]
pub struct ImGui<S: Style> {
  pub style: S,
  pub select: Select,
  gui: Arc<ImGuiImpl<S>>,
}

impl<S: Style> ImGui<S> {
  /// Create a new gui object.
  ///
  /// Also creates an [Select](select/struct.Select.html) instance, which may be used outside the gui.
  ///
  /// # Arguments
  /// * `device` - vulkanism device handle
  /// * `wnd` - vulkanism window handle
  /// * `target` - vulkan image on which the gui is rendered
  /// * `mem` - vulkanism memory manager on which additional buffers and textures are allocated
  pub fn new(device: &vk::device::Device, wnd: &vk::wnd::Window, target: vk::Image, mut mem: vk::mem::Mem) -> Self {
    // we need dpi and size of window to get the render target extent
    let dpi = wnd.window.get_hidpi_factor();
    let extent = wnd.window.get_inner_size().unwrap();
    let extent: vk::Extent2D = vk::Extent2D::build()
      .set((extent.width * dpi) as u32, (extent.height * dpi) as u32)
      .into();

    // we temporarily create a command pool to create the default font
    let cmds = vk::cmd::CmdPool::new(device.handle, device.queues[0].family).unwrap();
    let font = Arc::new(font::dejavu_mono::new(device.handle, mem.clone(), device.queues[0].handle, &cmds));

    // we have a single uniform buffer with the viewport dimensions
    // this ub is shared by all gui components
    let mut ub_viewport = vk::NULL_HANDLE;
    vk::mem::Buffer::new(&mut ub_viewport)
      .uniform_buffer(2 * std::mem::size_of::<f32>() as vk::DeviceSize)
      .devicelocal(false)
      .bind(&mut mem.alloc, vk::mem::BindType::Block)
      .unwrap();

    {
      let mut map = mem.alloc.get_mapped(Handle::Buffer(ub_viewport)).unwrap();
      let data = map.as_slice_mut::<u32>();
      data[0] = extent.width as u32;
      data[1] = extent.height as u32;
    }

    // gui renderpass and framebuffer
    // we reuse the color target from outside the gui
    // the gui components are then rendered with alpha blending
    let draw = {
      let rp = vk::pass::Renderpass::build(device.handle)
        .attachment(
          0,
          vk::AttachmentDescription::build()
            .format(vk::FORMAT_B8G8R8A8_UNORM)
            .initial_layout(vk::IMAGE_LAYOUT_COLOR_ATTACHMENT_OPTIMAL)
            .load_op(vk::ATTACHMENT_LOAD_OP_LOAD),
        )
        .subpass(
          0,
          vk::SubpassDescription::build().bindpoint(vk::PIPELINE_BIND_POINT_GRAPHICS).color(0),
        )
        .dependency(vk::SubpassDependency::build().external(0))
        .create()
        .unwrap();

      let fb = Mutex::new(
        vk::pass::Framebuffer::build_from_pass(&rp, &mut mem.alloc)
          .extent(extent)
          .target(0, target)
          .create(),
      );

      Pass {
        rp,
        fb,
        draw: Mutex::new(DrawPass::new()),
      }
    };

    // cached pipelines are created at startup
    let pipes = Mutex::new(Pipelines::new(device.handle, draw.rp.pass, 0, ub_viewport));

    // creates style resources
    let style = S::new(mem.clone(), draw.rp.pass, pipes.lock().unwrap().ds_viewport);

    // selection components
    let select = Select::new(device.handle, extent, dpi, mem.clone(), pipes.lock().unwrap().ds_viewport);

    Self {
      style,
      select,
      gui: Arc::new(ImGuiImpl {
        device: device.handle,
        mem,

        draw,
        ub_viewport: Mutex::new(ub_viewport),

        pipes,

        scr: Mutex::new(None),

        font,
      }),
    }
  }

  /// Get the vulkan device for which the gui was created
  pub fn get_device(&self) -> vk::Device {
    self.gui.device
  }
  /// Get the vulkanism memory manager of the gui
  pub fn get_mem(&self) -> vk::mem::Mem {
    self.gui.mem.clone()
  }
  /// Get the gui's default font
  pub fn get_font(&self) -> Arc<Font> {
    self.gui.font.clone()
  }
  /// Get the specified pipeline from the gui's cached pipelines
  ///
  /// # Arguments
  /// * `id` - Pipeline identifier

  pub fn get_pipes<'a>(&'a self) -> std::sync::MutexGuard<'a, Pipelines> {
    self.gui.pipes.lock().unwrap()
  }

  /// Get the gui's draw pass
  ///
  ///
  pub fn get_drawpass<'a>(&'a self) -> std::sync::MutexGuard<'a, DrawPass> {
    self.gui.draw.draw.lock().unwrap()
  }

  /// Resize the gui
  ///
  /// Resizing creates a new framebuffer object with the specified vulkan image.
  /// Updates the interal uniform buffer with the viewport.
  ///
  /// # Arguments
  /// * `size` - The new size of the gui
  /// * `target` - The new render target image for the gui
  pub fn resize(&mut self, size: vk::Extent2D, target: vk::Image) {
    let mut mem = self.gui.mem.clone();
    *self.gui.draw.fb.lock().unwrap() = vk::pass::Framebuffer::build_from_pass(&self.gui.draw.rp, &mut mem.alloc)
      .extent(size)
      .target(0, target)
      .create();

    self.select.resize(size);

    let ub = *self.gui.ub_viewport.lock().unwrap();
    let mut map = mem.alloc.get_mapped(Handle::Buffer(ub)).unwrap();
    let data = map.as_slice_mut::<u32>();
    data[0] = size.width as u32;
    data[1] = size.height as u32;
  }

  /// Handle window events
  ///
  /// Also calls [Select::handle_events](select/struct.Select.html#method.handle_events) for the gui`s object selection manager.
  ///
  /// # Arguments
  /// * `e` - Event to be handled
  pub fn handle_events(&mut self, e: &vk::winit::Event) {
    self.select.handle_events(e);
    if let Some(scr) = self.gui.scr.lock().unwrap().as_mut() {
      scr.push_event(e);
    }
  }

  /// Begins gui rendering
  ///
  /// # Returns
  /// [Screen](window/struct.Screen.html) that submits components into command buffer and a [select query](select/struct.Query.html).
  pub fn begin(&mut self) -> Screen<S> {
    let fb = self.gui.draw.fb.lock().unwrap();
    match self.gui.scr.lock().unwrap().take() {
      Some(rw) => Screen::from_cached(self.clone(), fb.extent, fb.images[0], fb.begin(), fb.end(), rw),
      None => Screen::new(self.clone(), fb.extent, fb.images[0], fb.begin(), fb.end()),
    }
  }
  /// Finishs gui rendering
  ///
  /// This is automaticolly called when [Screen](window/struct.Screen.html) is pushed into a command buffer.
  ///
  /// The gui retains buffers for storing gui components and the select query so that no (re)allocations during rendering happen in continuous frames.
  ///
  /// # Arguments
  /// * `scr` - The [Screen](window/struct.Screen.html) returned from [begin](struct.ImGui.html#method.begin).
  pub fn end(&mut self, scr: Screen<S>) {
    self.gui.scr.lock().unwrap().replace(scr);
  }
}
