extern crate nobs_vulkanism_headless as vk;

use vk::pipes::build::*;

fn main() {
  ShaderUsage::new()
    .uses("src/sprites/sprites.vert")
    .uses("src/sprites/sprites.frag")
    .depends("src/sprites/pipeline.rs");
}

