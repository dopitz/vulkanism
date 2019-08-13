use super::ComponentStyle;
use super::Style;
use crate::rect::Rect;
use crate::select::SelectId;
use crate::ImGui;

#[derive(Clone, Copy)]
pub struct Fancy {}

impl Style for Fancy {
  type Component = ComponentFancy;

  fn new(_mem: vk::mem::Mem) -> Self {
    Self {}
  }
}

pub struct ComponentFancy {
  rect: Rect,
}

impl ComponentStyle<Fancy> for ComponentFancy {
  fn new(_gui: &ImGui<Fancy>) -> Self {
    Self {
      rect: Rect::from_rect(0, 0, 0, 0),
    }
  }
}

impl Component<Fancy> for ComponentFancy {
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
  fn draw<L: Layout>(&mut self, _wnd: &mut Window<L, Fancy>, _focus: &mut SelectId) -> Option<()> {
    None
  }
}

make_style!(Fancy);
