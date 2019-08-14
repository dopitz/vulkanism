mod pipeline;

use super::ComponentStyle;
use super::Style;
use crate::pipelines::PipePool;
use crate::rect::Rect;
use crate::select::SelectId;
use crate::ImGui;
use std::sync::Arc;
use std::sync::Mutex;
use vk::cmd::commands::DrawManaged;
use vk::cmd::commands::DrawVertices;
use vk::pass::MeshId;

pub use pipeline::Pipeline;

#[derive(Clone)]
pub struct Simple {
  pipe: Arc<Mutex<Pipeline>>,
  ds_viewport: vk::DescriptorSet,
}

impl Style for Simple {
  type Component = ComponentSimple;

  fn new(mem: vk::mem::Mem, pass_draw: vk::RenderPass, pass_select: vk::RenderPass, ds_viewport: vk::DescriptorSet) -> Self {
    let pipe = Arc::new(Mutex::new(Pipeline::new(mem.alloc.get_device(), pass_draw, 0, pass_select, 0)));
    Self { pipe, ds_viewport }
  }
}

pub struct ComponentSimple {
  mem: vk::mem::Mem,
  gui: ImGui<Simple>,
  ub: vk::Buffer,

  rect: Rect,
  draw_mesh: MeshId,
  select_mesh: MeshId,
  select_id_body: SelectId,
  select_id_border: SelectId,

  ds_style: vk::DescriptorSet,

  mouse_pressed: Option<EventButton>,
  dragging: Option<EventDrag>,
}

impl Drop for ComponentSimple {
  fn drop(&mut self) {
    self.mem.trash.push_buffer(self.ub);
    self.gui.style.pipe.lock().unwrap().pool.free_dset(self.ds_style);
  }
}

impl ComponentStyle<Simple> for ComponentSimple {
  fn new(gui: &ImGui<Simple>) -> Self {
    let mut mem = gui.get_mem();
    let mut ub = vk::NULL_HANDLE;
    vk::mem::Buffer::new(&mut ub)
      .uniform_buffer(std::mem::size_of::<pipeline::UbStyle>() as vk::DeviceSize)
      .devicelocal(false)
      .bind(&mut mem.alloc, vk::mem::BindType::Block)
      .unwrap();

    let (draw_mesh, select_mesh, ds_style) = {
      let (color, select) = gui.style.pipe.lock().unwrap().new_instance(gui.style.ds_viewport, ub);

      (
        gui.get_drawpass().new_mesh(
          color.bind_pipe,
          &[color.bind_ds_viewport, color.bind_ds_style],
          DrawManaged::new([].iter().into(), DrawVertices::with_vertices(12).into()),
        ),
        gui.select.new_mesh(
          select.bind_pipe,
          &[select.bind_ds_viewport, select.bind_ds_style],
          DrawManaged::new([].iter().into(), DrawVertices::with_vertices(12).into()),
        ),
        color.bind_ds_style.dset,
      )
    };

    let select_id_body = gui.select.new_ids(2);
    let select_id_border = select_id_body + 1;

    Self {
      mem,
      gui: gui.clone(),
      ub,

      rect: Rect::from_rect(0, 0, 0, 0),
      draw_mesh,
      select_mesh,
      select_id_body,
      select_id_border,

      ds_style,

      mouse_pressed: None,
      dragging: None,
    }
  }
}

#[derive(Debug, Clone, Copy)]
pub struct EventDrag {
  border: bool,
  begin: vkm::Vec2u,
  end: vkm::Vec2u,
  delta: vkm::Vec2i,
}

#[derive(Debug, Clone, Copy)]
pub struct EventButton {
  border: bool,
  button: vk::winit::ButtonId,
  position: vkm::Vec2u,
}

#[derive(Debug)]
pub enum Event {
  Pressed(EventButton),
  Released(EventButton),
  Drag(EventDrag),
}

impl Component<Simple> for ComponentSimple {
  fn rect(&mut self, rect: Rect) -> &mut Self {
    if self.rect != rect {
      self.rect = rect;
      let mut map = self.mem.alloc.get_mapped(vk::mem::Handle::Buffer(self.ub)).unwrap();
      let data = map.as_slice_mut::<pipeline::UbStyle>();
      data[0].position = rect.position;
      data[0].size = rect.size.into();
      data[0].bd_thickness = vec2!(10);
      data[0].id_body = 1; //self.select_id_body.into();
      data[0].id_border = 2; //self.select_id_border.into();

      //self
      //  .gui
      //  .select
      //  .rects()
      //  .update_rect(self.select_rect, rect.position, rect.size);
    }
    self
  }
  fn get_rect(&self) -> Rect {
    self.rect
  }

  fn get_size_hint(&self) -> vkm::Vec2u {
    self.rect.size
  }

  type Event = Event;
  fn draw<L: Layout>(&mut self, wnd: &mut Window<L, Simple>, focus: &mut SelectId) -> Option<Event> {
    // apply_layout should be called by the wrapping gui element
    let scissor = vk::cmd::commands::Scissor::with_rect(self.get_rect().into());
    wnd.push_draw(self.draw_mesh, scissor);
    wnd.push_select(self.select_mesh, scissor);

    // event handling
    let select_body = wnd.get_select_result().filter(|id| *id == self.select_id_body).is_some();
    let select_border = wnd.get_select_result().filter(|id| *id == self.select_id_border).is_some();

    // assume a drag event if a mouse button is already pressed
    let mut event = if self.mouse_pressed.is_some() {
      let drag = self.dragging.take().map_or_else(
        || EventDrag {
          border: self.mouse_pressed.as_ref().unwrap().border,
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

    for e in wnd.get_events() {
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
          if select_body || select_border {
            event = Some(Event::Released(EventButton {
              border: select_border,
              button: *button,
              position: self.gui.select.get_current_position(),
            }));
          }
        }
        vk::winit::Event::DeviceEvent {
          event: vk::winit::DeviceEvent::Button {
            button,
            state: vk::winit::ElementState::Pressed,
          },
          ..
        } if select_body || select_border => {
          let bt = EventButton {
            border: select_border,
            button: *button,
            position: self.gui.select.get_current_position(),
          };
          self.mouse_pressed = Some(bt);
          event = Some(Event::Pressed(bt));
        }
        vk::winit::Event::WindowEvent {
          event: vk::winit::WindowEvent::CursorMoved { position, .. },
          ..
        } if self.mouse_pressed.is_some() => {
          let pos = self.gui.select.logic_to_real_position(*position).into();

          let drag = self.dragging.take().map_or_else(
            || EventDrag {
              border: self.mouse_pressed.as_ref().unwrap().border,
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

    event
  }
}

make_style!(Simple);
