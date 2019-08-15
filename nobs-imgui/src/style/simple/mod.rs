mod pipeline;

use super::Style;
use super::StyleComponent;
use crate::pipelines::PipePool;
use crate::rect::Rect;
use crate::select::SelectId;
use crate::ImGui;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use vk::cmd::commands::DrawManaged;
use vk::cmd::commands::DrawVertices;
use vk::pass::MeshId;

pub use pipeline::Pipeline;

struct LUTEntry {
  style: pipeline::UbStyleLUT,
  ds: vk::DescriptorSet,
  ub: vk::Buffer,
}

#[derive(Clone)]
pub struct Simple {
  mem: vk::mem::Mem,
  pipe: Arc<Mutex<Pipeline>>,
  style_lut: Arc<Mutex<HashMap<String, LUTEntry>>>,
  ds_style_default: vk::DescriptorSet,
  ds_viewport: vk::DescriptorSet,
}

/// TODO: drop Simple

impl Style for Simple {
  type Component = SimpleComponent;
  type Template = pipeline::UbStyleLUT;

  fn new(mut mem: vk::mem::Mem, pass_draw: vk::RenderPass, pass_select: vk::RenderPass, ds_viewport: vk::DescriptorSet) -> Self {
    let pipe = Arc::new(Mutex::new(Pipeline::new(mem.alloc.get_device(), pass_draw, 0, pass_select, 0)));
    let style_lut = Arc::new(Mutex::new(HashMap::new()));

    let mut ub = vk::NULL_HANDLE;
    vk::mem::Buffer::new(&mut ub)
      .uniform_buffer(std::mem::size_of::<pipeline::UbStyleLUT>() as vk::DeviceSize)
      .devicelocal(false)
      .bind(&mut mem.alloc, vk::mem::BindType::Block)
      .unwrap();

    {
      let mut map = mem.alloc.get_mapped(vk::mem::Handle::Buffer(ub)).unwrap();
      let data = map.as_slice_mut::<pipeline::UbStyleLUT>();
      data[0].color = vec4!(0.0, 1.0, 0.0, 1.0);
      data[0].bd_color_inner = vec4!(0.1, 0.0, 0.4, 1.0);
      data[0].bd_color_outer = vec4!(1.0, 1.0, 1.0, 1.0);
      data[0].bd_thickness = vec2!(10);
    }

    let ds_style_default = pipe.lock().unwrap().new_style(ub);

    Self {
      mem,
      pipe,
      style_lut,
      ds_style_default,
      ds_viewport,
    }
  }

  fn set_style(&mut self, name: String, style: Self::Template) {
    let mut styles = self.style_lut.lock().unwrap();

    let (ds, ub) = if let Some(e) = styles.get(&name) {
      (e.ds, e.ub)
    } else {
      let mut ub = vk::NULL_HANDLE;
      vk::mem::Buffer::new(&mut ub)
        .uniform_buffer(std::mem::size_of::<pipeline::UbStyleLUT>() as vk::DeviceSize)
        .devicelocal(false)
        .bind(&mut self.mem.alloc, vk::mem::BindType::Block)
        .unwrap();

      (self.pipe.lock().unwrap().new_style(ub), ub)
    };

    let mut map = self.mem.alloc.get_mapped(vk::mem::Handle::Buffer(ub)).unwrap();
    let data = map.as_slice_mut::<pipeline::UbStyleLUT>();
    data[0] = style;

    styles.insert(name, LUTEntry { style, ds, ub });
  }
  fn get_style(&self, name: &str) -> Option<Self::Template> {
    self.style_lut.lock().unwrap().get(name).map(|e| e.style)
  }

  fn load_styles(&mut self, styles: HashMap<String, Self::Template>) {
    let styles = styles
      .into_iter()
      .map(|(k, style)| {
        (
          k,
          LUTEntry {
            style,
            ds: vk::NULL_HANDLE,
            ub: vk::NULL_HANDLE,
          },
        )
      })
      .collect::<HashMap<_, _>>();

    /// TODO: delete ds / ub
    self.style_lut.lock().unwrap().clear();

    /// TODO: create buffers
  }
}

pub struct SimpleComponent {
  mem: vk::mem::Mem,
  gui: ImGui<Simple>,
  ub: vk::Buffer,
  ub_data: pipeline::UbStyle,
  dirty: bool,

  draw_mesh: MeshId,
  select_mesh: MeshId,

  ds_style: vk::DescriptorSet,

  mouse_pressed: Option<EventButton>,
  dragging: Option<EventDrag>,
}

impl Drop for SimpleComponent {
  fn drop(&mut self) {
    self.mem.trash.push_buffer(self.ub);
    self.gui.style.pipe.lock().unwrap().pool.free_dset(self.ds_style);
  }
}

impl StyleComponent<Simple> for SimpleComponent {
  fn new(gui: &ImGui<Simple>) -> Self {
    let mut mem = gui.get_mem();
    let mut ub = vk::NULL_HANDLE;
    vk::mem::Buffer::new(&mut ub)
      .uniform_buffer(std::mem::size_of::<pipeline::UbStyle>() as vk::DeviceSize)
      .devicelocal(false)
      .bind(&mut mem.alloc, vk::mem::BindType::Block)
      .unwrap();

    let (draw_mesh, select_mesh, ds_style) = {
      let (color, select) = gui
        .style
        .pipe
        .lock()
        .unwrap()
        .new_instance(gui.style.ds_viewport, gui.style.ds_style_default, ub);

      (
        gui.get_drawpass().new_mesh(
          color.bind_pipe,
          &[color.bind_ds_viewport, color.bind_ds_style_lut, color.bind_ds_style],
          DrawManaged::new([].iter().into(), DrawVertices::with_vertices(6).instance_count(9).into()),
        ),
        gui.select.new_mesh(
          select.bind_pipe,
          &[select.bind_ds_viewport, color.bind_ds_style_lut, select.bind_ds_style],
          DrawManaged::new([].iter().into(), DrawVertices::with_vertices(6).instance_count(9).into()),
        ),
        color.bind_ds_style.dset,
      )
    };

    let select_id = gui.select.new_ids(9);
    let ub_data = pipeline::UbStyle {
      position: vec2!(0),
      size: vec2!(0),
      id_body: select_id.into(),
      id_bd_topleft: (select_id + 1).into(),
      id_bd_topright: (select_id + 2).into(),
      id_bd_bottomleft: (select_id + 3).into(),
      id_bd_bottomright: (select_id + 4).into(),
      id_bd_top: (select_id + 5).into(),
      id_bd_bottom: (select_id + 6).into(),
      id_bd_left: (select_id + 7).into(),
      id_bd_right: (select_id + 8).into(),
    };

    Self {
      mem,
      gui: gui.clone(),
      ub,
      ub_data,
      dirty: true,

      draw_mesh,
      select_mesh,

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

impl Component<Simple> for SimpleComponent {
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

  type Event = Event;
  fn draw<L: Layout>(&mut self, wnd: &mut Window<L, Simple>, focus: &mut SelectId) -> Option<Event> {
    // update the uniform buffer if size changed
    if self.dirty {
      let mut map = self.mem.alloc.get_mapped(vk::mem::Handle::Buffer(self.ub)).unwrap();
      let data = map.as_slice_mut::<pipeline::UbStyle>();
      data[0] = self.ub_data;
      self.dirty = false;
    }

    // apply_layout should be called by the wrapping gui element
    let scissor = vk::cmd::commands::Scissor::with_rect(self.get_rect().into());
    wnd.push_draw(self.draw_mesh, scissor);
    wnd.push_select(self.select_mesh, scissor);

    // event handling
    let select_body = wnd
      .get_select_result()
      .filter(|id| *id == SelectId::from(self.ub_data.id_body))
      .is_some();
    let select_border = wnd
      .get_select_result()
      .filter(|id| *id == SelectId::from(self.ub_data.id_bd_top))
      .is_some();

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
