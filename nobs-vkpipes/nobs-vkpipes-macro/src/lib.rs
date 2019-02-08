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

#[proc_macro]
pub fn shader(input: TokenStream) -> TokenStream {
  let b = consume_err!(shader::Builder::from_tokens(input), "Error while parsing shader macro arguments");

  let s = consume_err!(b.new(), "Error while compiling shader macro arguments");

  if !b.dump.is_empty() {
    consume_err!(s.dump(&b.dump), "Error while writing shader module to file");
  }

  s.write_module().parse().unwrap()
}

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
