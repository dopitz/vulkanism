macro_rules! make_style {
  ($name:ty) => {
    pub use crate::window::Component;
    pub use crate::window::Layout;
    pub use crate::window::Window;

    pub type Gui = crate::ImGui<$name>;
    pub type TextBox = crate::textbox::TextBox<$name>;
  };
}

pub mod empty;
pub mod fancy;
pub mod simple;

use crate::window::Component;
use crate::ImGui;

pub trait ComponentStyle<S: Style>: Component<S> {
  fn new(gui: &ImGui<S>) -> Self;
}

pub trait Style: Clone {
  type Component: ComponentStyle<Self>;

  fn new(mem: vk::mem::Mem, pass: vk::RenderPass, ds_viewport: vk::DescriptorSet) -> Self;
}
