mod pipeline;
mod simple;
mod simplecomponent;

pub use super::Style;
pub use super::StyleComponent;
pub use simplecomponent::SimpleComponent;

use super::*;
use pipeline::Pipeline;
use pipeline::UbStyle;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

#[derive(Debug, Clone, Copy)]
struct LUTEntry {
  style: UbStyle,
  ds: vk::DescriptorSet,
  ub: vk::Buffer,
}

struct Impl {
  mem: vk::mem::Mem,
  pipe: Pipeline,
  style_lut: HashMap<String, LUTEntry>,
  style_default: LUTEntry,
}

impl Drop for Impl {
  fn drop(&mut self) {
    for ub in self.style_lut.iter().map(|(_, e)| e.ub) {
      self.mem.trash.push_buffer(ub);
    }
    self.mem.trash.push_buffer(self.style_default.ub);
  }
}

#[derive(Clone)]
pub struct Simple {
  im: Arc<Mutex<Impl>>,
  ds_viewport: vk::DescriptorSet,
}

impl Simple {
  fn lock<'a>(&'a self) -> std::sync::MutexGuard<'a, Impl> {
    self.im.lock().unwrap()
  }
}

impl Style for Simple {
  type Component = SimpleComponent;
  type Template = UbStyle;

  fn new(mut mem: vk::mem::Mem, pass_draw: vk::RenderPass, pass_select: vk::RenderPass, ds_viewport: vk::DescriptorSet) -> Self {
    let pipe = Pipeline::new(mem.alloc.get_device(), pass_draw, 0, pass_select, 0);
    let style_lut = HashMap::new();
    let style_default = {
      let style = UbStyle {
        color: vec4!(0.0, 0.3, 1.0, 0.2),
        bd_color_inner: vec4!(0.1, 0.0, 0.8, 0.8),
        bd_color_outer: vec4!(0.3, 0.1, 1.0, 1.0),
        bd_thickness: vec2!(10),
      };

      let mut ub = vk::NULL_HANDLE;
      vk::mem::Buffer::new(&mut ub)
        .uniform_buffer(std::mem::size_of::<UbStyle>() as vk::DeviceSize)
        .devicelocal(false)
        .bind(&mut mem.alloc, vk::mem::BindType::Block)
        .unwrap();

      let mut map = mem.alloc.get_mapped(vk::mem::Handle::Buffer(ub)).unwrap();
      let data = map.as_slice_mut::<UbStyle>();
      data[0] = style;

      LUTEntry {
        style,
        ds: pipe.new_style(ub),
        ub,
      }
    };

    Self {
      im: Arc::new(Mutex::new(Impl {
        mem,
        pipe,
        style_lut,
        style_default,
      })),
      ds_viewport,
    }
  }

  fn set_style(&mut self, name: String, style: Self::Template) {
    let mut im = self.im.lock().unwrap();

    let (ds, ub) = if let Some(e) = im.style_lut.get(&name) {
      (e.ds, e.ub)
    } else {
      let mut ub = vk::NULL_HANDLE;
      vk::mem::Buffer::new(&mut ub)
        .uniform_buffer(std::mem::size_of::<UbStyle>() as vk::DeviceSize)
        .devicelocal(false)
        .bind(&mut im.mem.alloc, vk::mem::BindType::Block)
        .unwrap();

      (im.pipe.new_style(ub), ub)
    };

    im.mem.alloc.get_mapped(vk::mem::Handle::Buffer(ub)).unwrap().host_to_device(&style);

    im.style_lut.insert(name, LUTEntry { style, ds, ub });
  }
  fn get_style(&self, name: &str) -> Option<Self::Template> {
    self.im.lock().unwrap().style_lut.get(name).map(|e| e.style)
  }

  fn load_styles(&mut self, styles: HashMap<String, Self::Template>) {
    // delete old descriptors and buffers
    let mut im = self.im.lock().unwrap();

    for (ds, ub) in im.style_lut.values().map(|e| (e.ds, e.ub)) {
      im.pipe.pool_lut.free_dset(ds);
      im.mem.trash.push_buffer(ub);
    }

    let mut styles = styles
      .into_iter()
      .map(|(k, style)| {
        (
          k,
          LUTEntry {
            style,
            ds: vk::NULL_HANDLE,
            ub: vk::NULL_HANDLE,
          },
        )
      })
      .collect::<HashMap<_, _>>();

    let mut builder = vk::mem::Resource::new();
    for (_, e) in styles.iter_mut() {
      builder = builder
        .new_buffer(&mut e.ub)
        .uniform_buffer(std::mem::size_of::<UbStyle>() as vk::DeviceSize)
        .devicelocal(false)
        .submit()
    }
    builder.bind(&mut im.mem.alloc, vk::mem::BindType::Block).unwrap();

    for (_, e) in styles.iter_mut() {
      im.mem
        .alloc
        .get_mapped(vk::mem::Handle::Buffer(e.ub))
        .unwrap()
        .host_to_device(&e.style);
      e.ds = im.pipe.new_style(e.ub);
    }

    im.style_lut = styles;
  }
}

make_style!(Simple);

pub fn get_default_styles() -> HashMap<String, UbStyle> {
  let mut styles = HashMap::new();
  styles.insert(
    "Window".to_owned(),
    UbStyle {
      color: vec4!(0.1, 0.05, 0.3, 0.6),
      bd_color_inner: vec4!(0.1, 0.1, 0.3, 1.0),
      bd_color_outer: vec4!(0.1, 0.05, 0.3, 1.0),
      bd_thickness: vec2!(3),
    },
  );
  styles.insert(
    "WindowBorderless".to_owned(),
    UbStyle {
      color: vec4!(0.1, 0.05, 0.3, 0.6),
      bd_color_inner: vec4!(0.0),
      bd_color_outer: vec4!(0.0),
      bd_thickness: vec2!(0),
    },
  );
  styles.insert(
    "WindowHeading".to_owned(),
    UbStyle {
      color: vec4!(0.1, 0.1, 0.3, 0.6),
      bd_color_inner: vec4!(0.1, 0.1, 0.3, 0.6),
      bd_color_outer: vec4!(0.1, 0.1, 0.3, 0.6),
      bd_thickness: vec2!(10),
    },
  );
  styles.insert(
    "TextBox".to_owned(),
    UbStyle {
      color: vec4!(1.0, 0.3, 1.0, 0.9),
      bd_color_inner: vec4!(0.1, 0.0, 0.8, 0.8),
      bd_color_outer: vec4!(0.3, 0.1, 1.0, 1.0),
      bd_thickness: vec2!(10),
    },
  );
  styles.insert(
    "TextBoxBorderless".to_owned(),
    UbStyle {
      color: vec4!(1.0, 0.3, 1.0, 0.9),
      bd_color_inner: vec4!(0.0),
      bd_color_outer: vec4!(0.0),
      bd_thickness: vec2!(0),
    },
  );
  styles
}
