use crate::component::stream::StreamCache;
use crate::component::Stream;
use crate::pipelines::Pipelines;
use crate::select::SelectPass;
use crate::style::Style;
use std::sync::Arc;
use std::sync::Mutex;
use vk::builder::Buildable;
use vk::cmd::stream::*;
use vk::mem::Handle;
use vk::pass::DrawPass;
use vk::winit;

struct Pass {
  rp: vk::pass::Renderpass,
  fb: Mutex<vk::pass::Framebuffer>,
  draw: Mutex<DrawPass>,
}

struct ImGuiImpl {
  device: vk::Device,
  mem: vk::mem::Mem,

  draw: Pass,
  ub_viewport: Mutex<vk::Buffer>,

  pipes: Mutex<Pipelines>,

  stream_cache: Mutex<StreamCache>,
}

impl Drop for ImGuiImpl {
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
  pub select: SelectPass,
  gui: Arc<ImGuiImpl>,
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
    let dpi = wnd.window.scale_factor();
    let extent = wnd.window.inner_size();
    let extent: vk::Extent2D = vk::Extent2D::build()
      .set((extent.width as f64 * dpi) as u32, (extent.height as f64 * dpi) as u32)
      .into();

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

    // selection components
    let select = SelectPass::new(device.handle, extent, dpi, mem.clone());

    // creates style resources
    let style = S::new(
      device,
      mem.clone(),
      draw.rp.pass,
      select.get_renderpass(),
      pipes.lock().unwrap().ds_viewport,
    );

    let stream_cache = Mutex::new(StreamCache::new(&mem));

    Self {
      style,
      select,
      gui: Arc::new(ImGuiImpl {
        device: device.handle,
        mem,

        draw,
        ub_viewport: Mutex::new(ub_viewport),

        pipes,

        stream_cache,
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
  /// Get the gui's cache of pipelines
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

  pub fn begin<'a>(&mut self, event: Option<&'a winit::event::Event<i32>>) -> Stream<'a, S, ()> {
    if let Some(event) = event.as_ref() {
      self.select.handle_event(event);
    }

    let fb = self.gui.draw.fb.lock().unwrap();
    self
      .gui
      .stream_cache
      .lock()
      .unwrap()
      .into_stream(self.clone(), fb.extent, fb.images[0], fb.begin(), fb.end(), event)
  }

  pub fn end<'a, R: std::fmt::Debug>(&'a mut self, mut s: Stream<'a, S, R>, cs: Option<CmdBuffer>) -> Option<CmdBuffer> {
    let cs = cs.map(|cs| cs.push_mut(&mut s));
    self.gui.stream_cache.lock().unwrap().recover(s);
    cs
  }
}
