use std::sync::Arc;

use vk;

use crate::ImGui;

pub trait Component : vk::cmd::commands::StreamPush {
  fn add_compontent(&mut self, wnd: &WindowComponents);
}

pub struct Window {
  pub ub_viewport: vk::Buffer,
}

impl Drop for Window {
  fn drop(&mut self) {

  }
}



pub struct WindowComponents<'a> {
  pub ub_viewport: vk::Buffer,

  compontents: Vec<&'a mut Component>,
}

impl<'a> WindowComponents<'a> {
  pub fn push(mut self, c: &'a mut Component) -> Self {
    c.add_compontent(&self);
    self.compontents.push(c);
    self
  }

  pub fn clear(&mut self) {
    self.compontents.clear();
  }
}

impl<'a> vk::cmd::commands::StreamPush for WindowComponents<'a> {
  fn enqueue(&self, cs: vk::cmd::Stream) -> vk::cmd::Stream {
    let mut cs = cs;
    for c in self.compontents.iter() {
      cs = c.enqueue(cs);
    }
    cs
  }
}


impl Window {
//  pub fn new(gui: Arc<ImGui>, _text: &str) -> Self {
//    let mut ub = vk::NULL_HANDLE;
//    vk::mem::Buffer::new(&mut ub)
//      .uniform_buffer(2 * std::mem::size_of::<f32>() as vk::DeviceSize)
//      .devicelocal(false)
//      .bind(&mut gui.alloc.clone(), vk::mem::BindType::Block)
//      .unwrap();
//
//    Self { ub }
//  }
//
//  pub fn set_position(&self, x: u32, y: u32) {
//  }
//
//  pub fn resize(&self, w: u32, h: u32) {
//    let mut map = self.gui.alloc.get_mapped(self.ub).unwrap();
//    let data = map.as_slice_mut::<u32>();
//    data[0] = sc.extent.width as u32;
//    data[1] = sc.extent.height as u32;
//  }
}
