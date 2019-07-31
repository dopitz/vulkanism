use crate::font::*;
use crate::rect::Rect;
use crate::select::SelectId;
use crate::text::Text;
use crate::window::Component;
use crate::window::Layout;
use crate::window::Window;
use crate::ImGui;
use vk::pass::MeshId;

#[derive(Debug)]
pub enum Event {
  Clicked,
  MouseOver,
  Changed,
}

pub struct TextBox {
  rect: Rect,
  text: Text,
  select_rect: usize,
  select_mesh: MeshId,
  select_id: SelectId,
}

impl Drop for TextBox {
  fn drop(&mut self) {
    self.get_gui().select.rects().remove(self.select_rect);
  }
}

impl TextBox {
  pub fn new(gui: &ImGui) -> Self {
    let rect = Rect::from_rect(0, 0, 200, 20);
    let text = Text::new(gui);
    let select_rect = gui.select.rects().new_rect(vec2!(0), vec2!(0));
    let select_id = gui.select.rects().get_select_id(select_rect);
    let select_mesh = gui.select.rects().get_mesh(select_rect);
    Self {
      rect,
      text,
      select_rect,
      select_mesh,
      select_id,
    }
  }

  pub fn get_gui(&self) -> ImGui {
    self.text.get_gui()
  }

  pub fn text(&mut self, text: &str) -> &mut Self {
    self.text.text(text);
    self
  }
  pub fn get_text(&self) -> String {
    self.text.get_text()
  }

  pub fn typeset(&mut self, ts: TypeSet) -> &mut Self {
    self.text.typeset(ts);
    self
  }
  pub fn get_typeset(&self) -> TypeSet {
    self.text.get_typeset()
  }
}

impl Component for TextBox {
  fn rect(&mut self, rect: Rect) -> &mut Self {
    if self.rect != rect {
      self
        .get_gui()
        .select
        .rects()
        .update_rect(self.select_rect, rect.position, rect.size);
      self.text.position(rect.position);
      self.rect = rect;
    }
    self
  }
  fn get_rect(&self) -> Rect {
    self.rect
  }

  fn get_size_hint(&self) -> vkm::Vec2u {
    let w = 0;
    let h = self.get_text().lines().count() as f32 * self.get_typeset().line_spacing * self.get_typeset().size as f32;
    vec2!(w, h as u32)
  }

  fn get_mesh(&self) -> MeshId {
    self.text.get_mesh()
  }

  fn get_select_mesh(&self) -> Option<MeshId> {
    Some(self.select_mesh)
  }

  type Event = Event;
  fn draw<T: Layout>(&mut self, wnd: &mut Window<T>) -> Option<Event> {
    wnd.push(self);

    let mut clicked = false;
    for e in wnd.get_events() {
      match e {
        vk::winit::Event::DeviceEvent {
          event: vk::winit::DeviceEvent::Button {
            button,
            state: vk::winit::ElementState::Pressed,
          },
          ..
        } if *button == 1 => clicked = true,
        _ => (),
      }
    }

    if clicked {
      wnd
        .get_select_result()
        .and_then(|id| if id == self.select_id { Some(Event::Clicked) } else { None })
    } else {
      None
    }
  }
}
