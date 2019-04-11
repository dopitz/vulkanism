extern crate nobs_vulkanism_headless as vk;

use vk::pipes::build::*;

fn main() {
  ShaderUsage::new()
    .uses("src/text/text.vert")
    .uses("src/text/text.frag")
    .depends("src/text/mod.rs");
}

