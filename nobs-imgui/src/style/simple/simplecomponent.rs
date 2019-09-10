use super::pipeline::Ub;
use super::*;
use crate::rect::Rect;
use crate::select::SelectId;
use crate::style::event::*;
use crate::window::Layout;
use crate::window::Screen;
use crate::window::Size;
use crate::ImGui;
use vk::cmd::commands::DrawManaged;
use vk::cmd::commands::DrawVertices;
use vk::pass::MeshId;

pub struct SimpleComponent {
  mem: vk::mem::Mem,
  gui: ImGui<Simple>,
  ub: vk::Buffer,
  ub_data: Ub,
  dirty: bool,

  style: String,
  bd_thickness: vkm::Vec2i,
  movable: bool,
  resizable: bool,

  draw_mesh: MeshId,
  select_mesh: MeshId,
  ds: vk::DescriptorSet,

  has_focus: bool,
  mouse_pressed: Option<EventButton>,
  dragging: Option<EventDrag>,
  movedelta: Option<vkm::Vec2i>,
}

impl Drop for SimpleComponent {
  fn drop(&mut self) {
    self.mem.trash.push_buffer(self.ub);
    self.gui.style.lock().pipe.pool.free_dset(self.ds);
  }
}

impl StyleComponent<Simple> for SimpleComponent {
  fn new(gui: &ImGui<Simple>, style: String, movable: bool, resizable: bool) -> Self {
    let mut mem = gui.get_mem();
    let mut ub = vk::NULL_HANDLE;
    vk::mem::Buffer::new(&mut ub)
      .uniform_buffer(std::mem::size_of::<super::pipeline::UbStyle>() as vk::DeviceSize)
      .devicelocal(false)
      .bind(&mut mem.alloc, vk::mem::BindType::Block)
      .unwrap();

    let mut sim = gui.style.im.lock().unwrap();

    let lutentry = *sim.style_lut.get(&style).unwrap_or(&sim.style_default);
    let bd_thickness = lutentry.style.bd_thickness;

    let (draw_mesh, select_mesh, ds) = {
      let (color, select) = sim.pipe.new_instance(gui.style.ds_viewport, lutentry.ds, ub);

      (
        gui.get_drawpass().new_mesh(
          color.bind_pipe,
          &[color.bind_ds_viewport, color.bind_ds_style, color.bind_ds],
          DrawManaged::new([].iter().into(), DrawVertices::with_vertices(6).instance_count(9).into()),
        ),
        gui.select.new_mesh(
          select.bind_pipe,
          &[color.bind_ds_viewport, color.bind_ds_style, color.bind_ds],
          DrawManaged::new([].iter().into(), DrawVertices::with_vertices(6).instance_count(9).into()),
        ),
        color.bind_ds.dset,
      )
    };

    let select_id = gui.select.new_ids(9);
    let ub_data = Ub {
      position: vec2!(0),
      size: vec2!(0),
      id_body: select_id.into(),
    };

    Self {
      mem,
      gui: gui.clone(),
      ub,
      ub_data,
      dirty: true,

      style,
      bd_thickness,
      movable,
      resizable,

      draw_mesh,
      select_mesh,

      ds,

      has_focus: false,
      mouse_pressed: None,
      dragging: None,
      movedelta: None,
    }
  }

  fn change_style(&mut self, style: &str, movable: bool, resizable: bool) {
    let sim = self.gui.style.lock();
    self.style = style.to_owned();
    let lutentry = sim.style_lut.get(&self.style).unwrap_or(&sim.style_default);
    self.bd_thickness = lutentry.style.bd_thickness;

    let (color, select) = sim.pipe.get_bindings(self.gui.style.ds_viewport, lutentry.ds, self.ds);

    self.gui.get_drawpass().update_mesh(
      self.draw_mesh,
      None,
      &[Some(color.bind_ds_viewport), Some(color.bind_ds_style), Some(color.bind_ds)],
      &[],
      None,
    );
    self.gui.select.update_mesh(
      self.select_mesh,
      None,
      &[Some(select.bind_ds_viewport), Some(select.bind_ds_style), Some(select.bind_ds)],
      &[],
      None,
    );

    self.movable = movable;
    self.resizable = resizable;
  }

  fn get_client_rect(&self) -> Rect {
    let mut rect = self.get_rect();
    rect.position += self.bd_thickness;
    rect.size = vkm::Vec2::clamp(rect.size.into() - self.bd_thickness * 2, vec2!(0), rect.size.into()).into();
    rect
  }

  fn get_padded_size(&self, size: vkm::Vec2u) -> vkm::Vec2u {
    size + self.bd_thickness.into() * 2
  }

  fn focus(&mut self, focus: bool) {
    self.has_focus = focus;
  }
  fn has_focus(&self) -> bool {
    self.has_focus
  }
}

impl Size for SimpleComponent {
  fn rect(&mut self, rect: Rect) -> &mut Self {
    if self.get_rect() != rect {
      self.ub_data.position = rect.position;
      self.ub_data.size = rect.size.into();
      self.dirty = true;
    }
    self
  }
  fn get_rect(&self) -> Rect {
    Rect::new(self.ub_data.position, self.ub_data.size.into())
  }

  fn get_size_hint(&self) -> vkm::Vec2u {
    self.ub_data.size.into()
  }
}

impl Component<Simple> for SimpleComponent {
  type Event = Event;
  fn draw<L: Layout>(&mut self, screen: &mut Screen<Simple>, layout: &mut L, _focus: &mut SelectId) -> Option<Event> {
    // update the uniform buffer if size changed
    if self.dirty {
      let mut map = self.mem.alloc.get_mapped(vk::mem::Handle::Buffer(self.ub)).unwrap();
      let data = map.as_slice_mut::<Ub>();
      data[0] = self.ub_data;
      self.dirty = false;
    }

    // apply_layout should be called by the wrapping gui element
    let scissor = layout.get_scissor(self.get_rect());
    screen.push_draw(self.draw_mesh, scissor);
    screen.push_select(self.select_mesh, scissor);

    // event handling
    let clicked = screen
      .get_select_result()
      .and_then(|id| ClickLocation::from_id(self.ub_data.id_body, id.into()));

    // assume a drag event if a mouse button is already pressed
    let mut event = if self.mouse_pressed.is_some() {
      let drag = self.dragging.take().map_or_else(
        || EventDrag {
          location: self.mouse_pressed.as_ref().unwrap().location,
          begin: self.mouse_pressed.as_ref().unwrap().position,
          end: self.mouse_pressed.as_ref().unwrap().position,
          delta: vec2!(0),
        },
        |mut d| {
          d.delta = self.gui.select.get_current_position().into() - d.end.into();
          d.end = self.gui.select.get_current_position();
          d
        },
      );

      self.dragging = Some(drag);
      Some(Event::Drag(drag))
    } else {
      None
    };

    for e in screen.get_events() {
      match e {
        vk::winit::Event::DeviceEvent {
          event: vk::winit::DeviceEvent::Button {
            button,
            state: vk::winit::ElementState::Released,
          },
          ..
        } => {
          self.mouse_pressed = None;
          self.dragging = None;
          self.movedelta = None;
          if clicked.is_some() {
            event = Some(Event::Released(EventButton {
              location: *clicked.as_ref().unwrap(),
              button: *button,
              position: self.gui.select.get_current_position(),
            }));
          } else {
            self.has_focus = false;
          }
        }
        vk::winit::Event::DeviceEvent {
          event: vk::winit::DeviceEvent::Button {
            button,
            state: vk::winit::ElementState::Pressed,
          },
          ..
        } if clicked.is_some() => {
          let bt = EventButton {
            location: *clicked.as_ref().unwrap(),
            button: *button,
            position: self.gui.select.get_current_position(),
          };
          self.has_focus = true;
          self.mouse_pressed = Some(bt);
          self.movedelta = Some(self.gui.select.get_current_position().into() - self.get_rect().position);
          event = Some(Event::Pressed(bt));
        }
        vk::winit::Event::WindowEvent {
          event: vk::winit::WindowEvent::CursorMoved { position, .. },
          ..
        } if self.mouse_pressed.is_some() => {
          let pos = self.gui.select.logic_to_real_position(*position).into();

          let drag = self.dragging.take().map_or_else(
            || EventDrag {
              location: self.mouse_pressed.as_ref().unwrap().location,
              begin: self.mouse_pressed.as_ref().unwrap().position,
              end: self.mouse_pressed.as_ref().unwrap().position,
              delta: vec2!(0),
            },
            |mut d| {
              d.delta = pos.into() - d.end.into();
              d.end = pos;
              d
            },
          );

          self.dragging = Some(drag);
          event = Some(Event::Drag(drag));
        }
        _ => (),
      }
    }

    // moving and resizing
    match event.as_ref() {
      Some(Event::Drag(drag)) => {
        if drag.delta != vec2!(0) {
          let mut pos = self.get_rect().position;
          let mut size = self.get_rect().size.into();

          let mp = self.gui.select.get_current_position().into();
          if self.resizable {
            match drag.location {
              ClickLocation::TopLeft => {
                size = pos + size - mp;
                pos = mp;
              }
              ClickLocation::TopRight => {
                size = vec2!(mp.x - pos.x, pos.y + size.y - mp.y);
                pos = vec2!(pos.x, mp.y);
              }
              ClickLocation::BottomLeft => {
                size = vec2!(pos.x + size.x - mp.x, mp.y - pos.y);
                pos = vec2!(mp.x, pos.y);
              }
              ClickLocation::BottomRight => {
                size = mp - pos;
              }

              ClickLocation::Top => {
                size.y = pos.y + size.y - mp.y;
                pos.y = mp.y;
              }
              ClickLocation::Bottom => {
                size.y = mp.y - pos.y;
              }
              ClickLocation::Left => {
                size.x = pos.x + size.x - mp.x;
                pos.x = mp.x;
              }
              ClickLocation::Right => {
                size.x = mp.x - pos.x;
              }
              _ => {}
            }
          }

          match drag.location {
            ClickLocation::Body if self.movable => pos = mp - *self.movedelta.as_ref().unwrap(),
            _ => {}
          }

          if self.movable || self.resizable {
            event = Some(Event::Resize(Rect::new(pos, size.into())));
          }
        }
      }
      _ => {}
    };

    event
  }
}
