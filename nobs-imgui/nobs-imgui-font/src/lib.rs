extern crate nobs_vulkanism_headless as vk;
#[macro_use]
extern crate nobs_vkmath as vkm;
extern crate nobs_imgui_font_macro as fnt;

mod font;
mod typeset;

pub use font::Char;
pub use font::Font;
pub use typeset::TypeSet;
pub use typeset::FontChar;

pub mod dejavu {
  use crate::Char;
  use crate::Font;
  use vk::builder::*;

  fnt::make_font! {
    font = "fonts/DejaVuSans.ttf",
  }
}

pub mod dejavu_mono {
  use crate::Char;
  use crate::Font;
  use vk::builder::*;

  fnt::make_font! {
    font = "fonts/DejaVuSansMono.ttf",
  }
}

pub mod dejavu_serif {
  use crate::Char;
  use crate::Font;
  use vk::builder::*;

  fnt::make_font! {
    font = "fonts/DejaVuSerif.ttf",
  }
}
