extern crate nobs_vulkanism_headless as vk;
#[macro_use]
extern crate nobs_vkmath as vkm;

mod assets;
mod update;

pub mod image;
pub mod model;

pub use assets::AssetPool;
pub use assets::Asset;
pub use update::Update;
