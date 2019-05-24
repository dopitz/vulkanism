use crate::font::*;
use crate::rect::Rect;
use crate::text::Text;
use crate::window::Window;
use crate::ImGui;

use vk;
use vk::cmd;
use vk::cmd::commands as cmds;

pub struct TextBox {
  rect: cmds::Scissor,
  text: Text,
}

impl TextBox {
  pub fn new(gui: &ImGui) -> Self {
    let rect = cmds::Scissor::with_size(200, 20);
    let text = Text::new(gui);

    Self { rect, text }
  }

  pub fn text(&mut self, text: &str) -> &mut Self {
    self.text.text(text);
    self
  }

  pub fn font(&mut self, font: std::sync::Arc<Font>, size: u32) -> &mut Self {
    self.text.font(font, size);
    self
  }

  pub fn rect(&mut self, rect: Rect) -> &mut Self {
    self
  }

  pub fn get_text(&self) -> String {
    self.text.get_text()
  }
}

impl cmds::StreamPush for TextBox {
  fn enqueue(&self, cs: cmd::Stream) -> cmd::Stream {
    cs.push(&self.rect).push(&self.text)
  }
}

//impl crate::window::Component for Text {
//  fn add_compontent(&mut self, wnd: &mut Window) {
//    if self.ub_viewport != wnd.ub_viewport {
//      self.ub_viewport = wnd.ub_viewport;
//      pipe::DsViewport::write(wnd.device, self.ds_viewport.dset)
//        .ub_viewport(|b| b.buffer(self.ub_viewport))
//        .update();
//    }
//
//    let rect = wnd.get_next_bounds();
//    if self.position != rect.position {
//      self.position = rect.position;
//
//      let mut map = self.mem.alloc.get_mapped(self.ub).unwrap();
//      let data = map.as_slice_mut::<pipe::Ub>();
//      data[0].offset = rect.position;
//    }
//
//    self.update_vb();
//  }
//}
