//! # nobs-vkpipes
//! Compiles shaders from glsl and generates rust code from spv.
//!
//! This crate provides builder pattern implementations for vulkan pipeline creation and descriptor set updating.
//!
//! ## Example
//! This is a simple example that sets up a compute pipeline.
//!
//! All the magic happens in `vk::pipes::pipeline!` macro!
//! We define the pipeline with several comma separated fields, paths are always specified relative to the compilied crate's cargo.toml:
//!
//! See the reexported macros [pipeline](../nobs_vkpipes_macro/macro.pipeline.html) and [shader](../nobs_vkpipes_macro/macro.shader.html) for a list of configurable options.
//! ```rust
//! extern crate nobs_vulkanism as vk;
//! // IMPORTANT import these two crates with their original name
//! // (e.g. not extern crate nobs_vk as vk)
//! // Otherwise the code generation will genertate code
//! // that does not find symbols defined there
//! // You can still use this...
//!
//! // declare the module that will contain our pipeline
//! mod make_sequence {
//!   vk::pipes::pipeline!{
//!     dset_name[0] = "Dset",
//!     stage = {
//!       ty = "comp",
//!       glsl = "
//! #version 450
//! #extension GL_ARB_separate_shader_objects : enable
//!
//! const uint GROUP_SIZE = 512;
//!
//! layout(binding = 0) uniform ub {
//!   uint num_elems;
//!   uint i_first;
//!   uint i_step;
//! };
//!
//! layout(binding = 1) buffer b_out {
//!   uint bout[];
//! };
//!
//! layout(local_size_x = GROUP_SIZE) in;
//! void main() {
//!   // copy input values for group in shared memory
//!   uint gid = gl_GlobalInvocationID.x;
//!
//!   if (gid < num_elems) {
//!     bout[gid] = i_first + gid * i_step;
//!   }
//! }
//!         ",
//!     }
//!   }
//!
//!   // The code generation will not create types for e.g. the uniform buffer
//!   // If we want this, we need to do it our selves
//!   pub struct ub {
//!     pub num_elems: u32,
//!     pub i_first: u32,
//!     pub i_step: u32,
//!   }
//! }
//!
//! // create an instance of the pipeline
//! // uses the nobs_vk::DeviceExtensions to build the pipeline
//! //let p = make_sequence::build(&device_ext).new().unwrap();
//!
//! // update the descriptor set
//! //make_sequence::dset::write(&device.ext, ds)
//! //  .ub(|b| b.buffer(buf_ub))
//! //  .b_out(|b| b.buffer(buf_out))
//! //  .update();
//! ```

#[macro_use]
extern crate nobs_vk as vk;
extern crate nobs_vkpipes_macro;

// Codegeneration macros
pub use nobs_vkpipes_macro::pipeline;
pub use nobs_vkpipes_macro::shader;

pub mod pipeline;
pub use pipeline::builder::compute::Compute as ComputeBuilder;
pub use pipeline::builder::graphics::blend::AttachmentBuilder as BlendAttachment;
pub use pipeline::builder::graphics::blend::Builder as Blend;
pub use pipeline::builder::graphics::depth_stencil::Builder as DepthStencil;
pub use pipeline::builder::graphics::dynamic::Builder as Dynamic;
pub use pipeline::builder::graphics::input_assembly::Builder as InputAssembly;
pub use pipeline::builder::graphics::multisample::Builder as Multisample;
pub use pipeline::builder::graphics::raster::Builder as Raster;
pub use pipeline::builder::graphics::tesselation::Builder as Tesselation;
pub use pipeline::builder::graphics::vertex_input::Builder as VertexInput;
pub use pipeline::builder::graphics::viewport::Builder as Viewport;
pub use pipeline::builder::graphics::Graphics as GraphicsBuilder;

pub mod descriptor;
pub use descriptor::pool::Pool as DescriptorPool;

/// For usage in build.rs to automatically detect changes in glsl/spv files and force the recompilation of the rust source that references the shader.
///
/// ## Expample
/// An example build.rs. Every time src/my_shader.comp changes src/main.rs is flagged to recompile as well.
/// ```
/// extern crate nobs_vkpipes as vkpipes;
/// use vkpipes::build::ShaderUsage;
/// fn main() {
///   ShaderUsage::new().uses("src/my_shader.comp").depends("src/main.rs");
/// }
/// ```
pub mod build {
  /// Creates a dependency between a shader source and rust source
  pub struct ShaderUsage {}

  impl ShaderUsage {
    /// Creates a new mapping
    pub fn new() -> ShaderUsage {
      ShaderUsage {}
    }

    /// Specify a filename of a shader, that if changed triggers the recompilation of all dependent rust sources
    pub fn uses(&self, filename: &str) -> &Self {
      if std::path::Path::new(filename).exists() {
        println!("cargo:rerun-if-changed={}", filename);
      } else {
        println!(
          "cargo:warning=The file {} does not exists, but is listed as used shader resource",
          filename
        );
      }
      &self
    }

    /// Specify a filename of a rust source, that is recompiled if any of used shaders was changed
    pub fn depends(&self, filename: &str) -> &Self {
      if std::path::Path::new(filename).exists() {
        std::process::Command::new("sh")
          .arg("-c")
          .arg(format!("touch {}", filename))
          .output()
          .unwrap();
      } else {
        println!(
          "cargo:warning=The file {} does not exists, but is listed as shader usage dependency",
          filename
        );
      }
      &self
    }
  }
}
