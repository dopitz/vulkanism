//! Immediate gui as extension for [vulkanism](https://docs.rs/nobs-vulkanism)
//!
//! The gui's components are rendered with function calls that immediately push respectiove commands into a vulkan command buffer.
//! The same function calls will handle events and return respective results in case the user interacted with them.
//!
//! The gui declares a dependent crate nobs-imgui-font for sprite font creation.
//!
//! Several gui components are available:
//!  - [Textbox](textbox/struct.Textbox.html)
//!
//! On a special note we point to the object selection with [Select](select/struct.Select.html)
extern crate nobs_vulkanism as vk;
#[macro_use]
extern crate nobs_vkmath as vkm;
extern crate freetype;
extern crate nobs_imgui_font as font;

pub mod components;
pub mod pipelines;
pub mod rect;
pub mod select;
pub mod sprites;
pub mod style;
pub mod terminal;
pub mod window;

mod imgui;
pub use imgui::ImGui;
pub use select::Select;
