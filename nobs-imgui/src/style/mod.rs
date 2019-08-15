macro_rules! make_style {
  ($name:ty) => {
    pub use crate::window::Component;
    pub use crate::window::Layout;
    pub use crate::window::Window;

    pub type Gui = crate::ImGui<$name>;
    pub type TextBox = crate::textbox::TextBox<$name>;
  };
}

//pub mod empty;
//pub mod fancy;
pub mod simple;

use crate::window::Component;
use crate::ImGui;
use std::collections::HashMap;

pub trait Style: Clone {
  type Component: StyleComponent<Self>;
  type Template: Clone;

  fn new(mem: vk::mem::Mem, pass_draw: vk::RenderPass, pass_select: vk::RenderPass, ds_viewport: vk::DescriptorSet) -> Self;

  fn set_style(&mut self, name: String, style: Self::Template);
  fn get_style(&self, name: &str) -> Option<Self::Template>;

  fn load_styles(&mut self, styles: HashMap<String, Self::Template>);
}

pub trait StyleComponent<S: Style>: Component<S> {
  fn new(gui: &ImGui<S>) -> Self;
}
