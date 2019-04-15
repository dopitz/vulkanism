use std::sync::Arc;

use vk;

use crate::ImGui;

pub struct Window {
  gui: Arc<ImGui>,
  pub ub: vk::Buffer,
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
