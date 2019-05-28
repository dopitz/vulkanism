extern crate nobs_vulkanism_headless as vk;

use vk::pipes::build::*;

fn main() {
  ShaderUsage::new()
    .uses("src/sprite/sprite.vert")
    .uses("src/sprite/sprite.frag")
    .depends("src/sprite/pipeline.rs");
}

