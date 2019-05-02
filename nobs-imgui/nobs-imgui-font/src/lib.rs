extern crate proc_macro;
use std::fmt::Write;

use proc_macro::TokenStream;

#[proc_macro]
pub fn make_font(input: TokenStream) -> TokenStream {

  //let t = input.next();
  //.to_string();
  //s.chars().skip(1).take(s.len() - 2).collect();

  let mut s = String::new();
  write!(s, "fn aoeu() {{ println!(\"{}\"); }}", 1234);
  s.parse().unwrap()
}
