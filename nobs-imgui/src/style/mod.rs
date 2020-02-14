macro_rules! make_style {
  ($name:ty) => {
    pub type ThisStyle = $name;

    pub use crate::select;

    pub type Gui = crate::ImGui<$name>;
    pub use crate::rect::Rect;

    pub mod window {
      pub use crate::window::*;
      //TODO: pub type Window<L: Layout> = crate::window::Window<L, super::ThisStyle>;
      pub type Window<L> = crate::window::Window<L, super::ThisStyle>;
    }

    pub mod components {
      pub use crate::components::*;
      pub type TextBox<H = crate::components::textbox::HandlerReadonly> = crate::components::TextBox<super::ThisStyle, H>;
      pub type TextEdit = crate::components::TextEdit<super::ThisStyle>;
      pub type TextEditMultiline = crate::components::TextEditMultiline<super::ThisStyle>;
    }

    pub mod shell {
      pub use crate::shell::*;
      pub type Terminal = crate::shell::Terminal<super::ThisStyle>;
    }
  };
}

//pub mod empty;
//pub mod fancy;
pub mod simple;

use crate::font::TypeSet;
use crate::rect::Rect;
use crate::window::Component;
use crate::ImGui;
use std::collections::HashMap;

pub trait Style: Clone {
  type Component: StyleComponent<Self>;
  type Template: Clone;

  fn new(
    device: &vk::device::Device,
    mem: vk::mem::Mem,
    pass_draw: vk::RenderPass,
    pass_select: vk::RenderPass,
    ds_viewport: vk::DescriptorSet,
  ) -> Self;

  fn set_style(&mut self, name: String, style: Self::Template);
  fn get_style(&self, name: &str) -> Option<Self::Template>;

  fn load_styles(&mut self, styles: HashMap<String, Self::Template>);

  fn set_dpi(&mut self, dpi: f64);

  fn get_typeset_tini(&self) -> TypeSet;
  fn get_typeset_small(&self) -> TypeSet;
  fn get_typeset(&self) -> TypeSet;
  fn get_typeset_large(&self) -> TypeSet;
  fn get_typeset_huge(&self) -> TypeSet;
}

pub mod event {
  use crate::rect::Rect;

  #[derive(Debug, Clone, Copy)]
  pub enum ClickLocation {
    Body,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    Top,
    Bottom,
    Left,
    Right,
  }

  impl ClickLocation {
    pub fn from_id(id_body: u32, id: u32) -> Option<Self> {
      if id < id_body {
        return None;
      }
      match id - id_body {
        0 => Some(ClickLocation::Body),
        1 => Some(ClickLocation::TopLeft),
        2 => Some(ClickLocation::TopRight),
        3 => Some(ClickLocation::BottomLeft),
        4 => Some(ClickLocation::BottomRight),
        5 => Some(ClickLocation::Top),
        6 => Some(ClickLocation::Bottom),
        7 => Some(ClickLocation::Left),
        8 => Some(ClickLocation::Right),
        _ => None,
      }
    }
  }

  #[derive(Debug, Clone, Copy)]
  pub struct EventButton {
    pub location: ClickLocation,
    pub button: vk::winit::ButtonId,
    pub position: vkm::Vec2u,
    pub relative_pos: vkm::Vec2u,
  }

  #[derive(Debug, Clone, Copy)]
  pub struct EventDrag {
    pub start: EventButton,
    pub end: vkm::Vec2u,
    pub delta: vkm::Vec2i,
  }

  #[derive(Debug)]
  pub enum Event {
    Pressed(EventButton),
    Released(EventButton),
    Drag(EventDrag),
    Resize(Rect),
  }
}

pub trait StyleComponent<S: Style>: Component<S, Event = event::Event> {
  fn new(gui: &ImGui<S>, style: String, movable: bool, resizable: bool) -> Self;
  fn change_style(&mut self, style: &str, movable: bool, resizable: bool);

  fn get_client_rect(&self) -> Rect;
  fn get_padded_size(&self, size: vkm::Vec2u) -> vkm::Vec2u;

  fn focus(&mut self, focus: bool);
  fn has_focus(&self) -> bool;
}
