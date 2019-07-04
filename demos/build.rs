extern crate nobs_vulkanism as vk;

use vk::pipes::build::*;

fn main() {
  ShaderUsage::new().uses("src/make_sequence.comp").depends("src/headless.rs");
  ShaderUsage::new()
    .uses("src/triangle.vert")
    .uses("src/triangle.frag")
    .depends("src/triangle.rs");
  ShaderUsage::new()
    .uses("src/textured.vert")
    .uses("src/textured.frag")
    .depends("src/textured.rs");
  ShaderUsage::new()
    .uses("src/obj.vert")
    .uses("src/obj.frag")
    .depends("src/obj.rs");
}
