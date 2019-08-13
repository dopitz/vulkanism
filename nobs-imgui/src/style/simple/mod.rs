mod pipeline;

use super::ComponentStyle;
use super::Style;
use crate::rect::Rect;
use crate::select::SelectId;
use crate::ImGui;
use std::sync::Arc;
use std::sync::Mutex;

pub use pipeline::Pipeline;

#[derive(Clone)]
pub struct Simple {
  //pipe: Arc<Mutex<Pipeline>>,
}

impl Style for Simple {
  type Component = ComponentSimple;

  fn new(mut mem: vk::mem::Mem) -> Self {
    let device = mem.alloc.get_device();

    let mut ub = vk::NULL_HANDLE;
    vk::mem::Buffer::new(&mut ub)
      .uniform_buffer(std::mem::size_of::<pipeline::UbStyle>() as vk::DeviceSize)
      .devicelocal(false)
      .bind(&mut mem.alloc, vk::mem::BindType::Block)
      .unwrap();

    //let pipe = Pipeline::new(&pipes[PipeId::StyleSimple]);
    //pipe.update_dsets(device, ub);

    Self {  }
  }
}

pub struct ComponentSimple {
  rect: Rect,
}

impl ComponentStyle<Simple> for ComponentSimple {
  fn new(_gui: &ImGui<Simple>) -> Self {
    Self {
      rect: Rect::from_rect(0, 0, 0, 0),
    }
  }
}

impl Component<Simple> for ComponentSimple {
  fn rect(&mut self, rect: Rect) -> &mut Self {
    self.rect = rect;
    self
  }
  fn get_rect(&self) -> Rect {
    self.rect
  }

  fn get_size_hint(&self) -> vkm::Vec2u {
    self.rect.size
  }

  type Event = ();
  fn draw<L: Layout>(&mut self, _wnd: &mut Window<L, Simple>, _focus: &mut SelectId) -> Option<()> {
    None
  }
}

make_style!(Simple);
