use super::Style;
use super::ComponentStyle;
use crate::rect::Rect;
use crate::select::SelectId;
use crate::ImGui;

#[derive(Clone, Copy)]
pub struct Empty {
}

impl Style for Empty {
  type Component = ComponentEmpty;

  fn new(_mem: vk::mem::Mem, _pass_draw: vk::RenderPass, _pass_select: vk::RenderPass, _ds_viewport: vk::DescriptorSet) -> Self {
    Self {}
  }
}

pub struct ComponentEmpty {
  rect: Rect,
}

impl ComponentStyle<Empty> for ComponentEmpty {
  fn new(_gui: &ImGui<Empty>) -> Self {
    Self {
      rect: Rect::from_rect(0, 0, 0, 0),
    }
  }
}

impl Component<Empty> for ComponentEmpty {
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
  fn draw<L: Layout>(&mut self, _wnd: &mut Window<L, Empty>, _focus: &mut SelectId) -> Option<()> {
    None
  }
}

make_style!(Empty);
