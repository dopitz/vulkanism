extern crate nobs_vulkanism_headless as vk;

use vk::pipes::build::*;

fn main() {
  ShaderUsage::with_prefix("src/sprites")
    .uses("sprites.vert")
    .uses("sprites.frag")
    .depends("pipeline.rs");

  ShaderUsage::with_prefix("src/select/rects")
    .uses("rects.vert")
    .uses("rects.frag")
    .depends("pipeline.rs");

  ShaderUsage::with_prefix("src/style/simple/pipeline")
    .uses("border.vert")
    .uses("borderless.vert")
    .uses("color.frag")
    .uses("select.frag")
    .depends("border.rs")
    .depends("borderless.rs")
    .depends("select.rs")
    .depends("selectborderless.rs");
}
