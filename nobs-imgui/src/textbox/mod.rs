use crate::font::*;
use crate::rect::Rect;
use crate::select::rects::RectId;
use crate::select::SelectId;
use crate::style::ComponentStyle;
use crate::style::Style;
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

pub struct TextBox<S: Style> {
  rect: Rect,
  text: Text<S>,
  select_rect: RectId,
  select_mesh: MeshId,
  select_id: SelectId,

  style: S::Component,
}

impl<S: Style> Drop for TextBox<S> {
  fn drop(&mut self) {
    self.get_gui().select.rects().remove(self.select_rect);
  }
}

impl<S: Style> TextBox<S> {
  pub fn new(gui: &ImGui<S>) -> Self {
    let rect = Rect::from_rect(0, 0, 0, 0);
    let text = Text::new(gui);
    let select_rect = gui.select.rects().new_rect(vec2!(0), vec2!(0));
    let select_id = gui.select.rects().get_select_id(select_rect);
    let select_mesh = gui.select.rects().get_mesh(select_rect);
    let mut style = S::Component::new(gui);
    Self {
      rect,
      text,
      select_rect,
      select_mesh,
      select_id,
      style,
    }
  }

  pub fn get_gui(&self) -> ImGui<S> {
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

impl<S: Style> Component<S> for TextBox<S> {
  fn rect(&mut self, rect: Rect) -> &mut Self {
    if self.rect != rect {
      self.style.rect(rect);
      // TODO: border thickness....
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

  type Event = Event;
  fn draw<L: Layout>(&mut self, wnd: &mut Window<L, S>, focus: &mut SelectId) -> Option<Event> {
    // style is resized along with the textbox
    let scissor = wnd.apply_layout(self);

    // draw and select
    self.style.draw(wnd, focus);
    wnd.push_draw(self.text.get_mesh(), scissor);
    wnd.push_select(self.select_mesh, scissor);

    // event handling
    let select_result = wnd.get_select_result();

    let mut clicked = false;
    for e in wnd.get_events() {
      match e {
        vk::winit::Event::DeviceEvent {
          event: vk::winit::DeviceEvent::Button {
            button,
            state: vk::winit::ElementState::Pressed,
          },
          ..
        } if *button == 1 => {
          clicked = select_result
            .and_then(|id| if id == self.select_id { Some(Event::Clicked) } else { None })
            .is_some();

          if clicked {
            *focus = self.select_id;
          }
        }
        _ => (),
      }

      if *focus == self.select_id {
        match e {
          vk::winit::Event::WindowEvent {
            event: vk::winit::WindowEvent::ReceivedCharacter(c),
            ..
          } => {
            // TODO: multiline flag?
            let mut c = *c;
            if c == '\r' {
              c = '\n';
            }
            self.text(&format!("{}{}", self.get_text(), c));
          }
          _ => (),
        }
      }
    }

    if clicked {
      Some(Event::Clicked)
    } else {
      None
    }
  }
}
