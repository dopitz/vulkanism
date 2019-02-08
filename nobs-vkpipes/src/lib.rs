//! # nobs-vkpipes
//! shader compilation and pipeline composing on top of nobs-vk.
//!
//! This crate provides builder pattern implementations for vulkan pipeline creation and descriptor set updating.
//!
//! ## Example
//! This is a simple example that sets up a compute pipeline.
//!
//! The main part happens in the `nobs_vkpipes::pipeline!` macro. We define the pipeline with several comma separated fields, paths are always specified relative to the compilied crate's cargo.toml:
//!  - `include = [str]`: [OPTIONAL] list of strings specifying include directories for the compilation of glsl shader files (for all specified stages).
//!    - Example: `include = ["src/global_shader_includes", "src/util"]`
//!  - `dump = str`: [OPTIONAL] filename to which the output of the codegeneration will be written.
//!    - Example: `dump = "dump/my_pipeline.rs"`
//!  - `dset_name[i32] = str`: [OPTIONAL] rename descriptor set with index 0. Since we can not specify descriptor set names in glsl, they are enumerated with Dset0, Dset1, Dset2.. if no name is specified for a descriptor set index.
//!    - Example: `dset_name[0] = "per_frame"`
//!  - `stage = {...}`: [MANDATORY] specifies a shader stage with parameters
//!    - Example: `ty = str`: [MANDATORY] type of the shader, must be one of ["comp", "vert", "tesc", "tese", "geom", "frag"]. The pipeline type (compute or graphics) is inferred from the specified stages.
//!    This means "comp" must NOT be mixed with any other stage type. Graphics pipelines need at least two stages specified ("vert" and "frag")
//!      - Example: `ty = "comp"`
//!    - `glsl = str` [MANDATORY] specifies either a shader file, or actual glsl shader code. If glsl is specified spv must not be specified.
//!    - `spv = str` [MANDATORY] specifies a compiled shader file. If spv is specified glsl must not be specified.
//!      - Example: `glsl = "src/my_shader.comp"`
//!
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

pub mod descriptor;
pub mod pipeline;

pub use descriptor::pool::Pool as DescriptorPool;
pub use descriptor::DsetLayout;
pub use pipeline::Pipeline;

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
