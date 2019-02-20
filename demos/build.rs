extern crate nobs_vulkanism as vk;

use vk::pipes::build::*;

fn main() {
  ShaderUsage::new().uses("src/make_sequence.comp").depends("src/compute.rs");
  ShaderUsage::new()
    .uses("src/fs_tri.vert")
    .uses("src/fs_tri.frag")
    .depends("src/triangle.rs");
}
