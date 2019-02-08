//! # nobs-vkpipes-macro
//! Proc-Macro crate for nobs-vkpipes
//!
//! This crate defines the actual proc macros for the pipeline and shader stage code generation
//!
//! ## Example
//! This is a simple example that sets up a compute pipeline.
//!
//! All the magic happens in `vk::pipes::pipeline!` macro!
//! We define the pipeline with several comma separated fields, paths are always specified relative to the compilied crate's cargo.toml:
//!
//! See [pipeline](macro.pipeline.html) and [shader](macro.shader.html) for complete list of supported options.
//! ```rust
//! extern crate nobs_vulkanism as vk;
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
//! ```

extern crate nobs_vk as vk;
extern crate proc_macro;

mod binding;
mod enums;
mod parse;
mod pipeline;
mod shader;
mod spirv;
mod usings;

use proc_macro::TokenStream;

macro_rules! consume_err {
  ($ex:expr, $msg:expr) => {
    match $ex {
      Err(e) => {
        return format!("compile_error!(\"{}\")", format!("{}: {}", $msg, e).replace("\"", "\\\""))
          .parse()
          .unwrap();
      }
      Ok(x) => x,
    }
  };
}

/// Generates code for a shader stage
///
/// The shader stage can be configured by a comma separated list of fields, described in the table below:
///
/// | specifier | type | mandatory | description | example |
/// | --------- | ---- | --------- | ----------- | ------- |
/// | `ty` | `str` | yes | Type of the shader, must be one of ["comp", "vert", "tesc", "tese", "geom", "frag"]. The pipeline type (compute or graphics) is inferred from the specified stages. | `ty = "comp"`|
/// | `glsl` | `str` | yes* | Specifies either a shader file, or actual glsl shader code. If glsl is specified spv must not be specified. | `glsl = "src/shader.comp"`|
/// | `spv` | `str` | yes* | Specifies a compiled shader file. If spv is specified glsl must not be specified. | `glsl = "src/shader.spv"`|
/// | `include` | `[str]` | no | List of strings specifying include directories if the shader is specified as glsl file. | `include = ["src/global", "src/util"]`|
/// | `vk_alias` | `str` | no | Alias of the import of the nobs-vk crate. Set to "vk" by default. | `vk_alias = "othervk"`|
/// | `vkpipes_alias` | `str` | no | Alias of the import of the nobs-vkpipes crate. Set to "vk::pipes" by default. | `vkpipes_alias = "otherpipes"`|
/// | `dump` | `str` | no | Filename to which the output of the code generation will be written. | `dump = "dump/my_pipeline.rs"`|
///
/// See [here] for whole pipeline configuration.
#[proc_macro]
pub fn shader(input: TokenStream) -> TokenStream {
  let b = consume_err!(shader::Builder::from_tokens(input), "Error while parsing shader macro arguments");

  let s = consume_err!(b.new(), "Error while compiling shader macro arguments");

  if !b.dump.is_empty() {
    consume_err!(s.dump(&b.dump), "Error while writing shader module to file");
  }

  s.write_module().parse().unwrap()
}

/// Generates code for a pipeline
///
/// A pipeline can be configured by a comma separated list of fields, described in the table below:
///
/// | specifier | type | mandatory | description | example |
/// | --------- | ---- | --------- | ----------- | ------- |
/// | `stage` | `{...}` | yes | Specifies a shader stage with parameters. Can be used multiple times to specify multiple stages. | see [shader](macro.shader.html) |
/// | `include` | `[str]` | no | List of strings specifying include directories for the compilation of glsl shader files. Include directories defined in the pipeline are used for all stages. | `include = ["src/global", "src/util"]`|
/// | `dset_name[i32]` | `[str]` | no | Rename descriptor set with index 0. Since we can not specify descriptor set names in glsl, they are enumerated with Dset0, Dset1, Dset2.. if no name is specified for a descriptor set index. | `dset_name[0] = "per_frame"`|
/// | `vk_alias` | `str` | no | Alias of the import of the nobs-vk crate. Set to "vk" by default. | `vk_alias = "othervk"`|
/// | `vkpipes_alias` | `str` | no | Alias of the import of the nobs-vkpipes crate. Set to "vk::pipes" by default. | `vkpipes_alias = "otherpipes"`|
/// | `dump` | `str` | no | Filename to which the output of the code generation will be written. | `dump = "dump/my_pipeline.rs"`|
///
/// The fields 'vk_alias' and 'vkpipes_alias' are important, if this crate is not used in conjuction with either of the vulkanism crates
/// and if the nobs-vkpipes crate was imported with an alias e.g.: 'extern crate nobs-vkpipes as pipes;'.
/// In these cases the code generation can not resolve the dependencies for the generated module code.
///
/// See the [module level documentation](index.html) for a complete example
#[proc_macro]
pub fn pipeline(input: TokenStream) -> TokenStream {
  let b = consume_err!(
    pipeline::Builder::from_tokens(input),
    "Error while parsing pipeline macro arguments"
  );
  let p = consume_err!(b.new(), "Error while compiling pipeline stages");

  if !b.dump.is_empty() {
    consume_err!(p.dump(&b.dump), "Error while writing pipeline module to file");
  }

  p.write_module().parse().unwrap()
}
