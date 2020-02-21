use crate::shell::command::args::Arg;
use crate::shell::command::args::ArgDesc;

/// Special Argument specifying the command name at `index == 0`
#[derive(Clone, Debug)]
pub struct Ident {
  desc: ArgDesc,
  variants: Vec<Vec<String>>,
  case: bool,
}

impl Ident {
  pub fn new(desc: ArgDesc, variants: &[&[&str]], case: bool) -> Self {
    Self {
      desc,
      variants: variants.iter().map(|v| v.iter().map(|s| s.to_string()).collect()).collect(),
      case,
    }
  }
}

impl Arg for Ident {
  fn get_desc<'a>(&'a self) -> &'a ArgDesc {
    &self.desc
  }

  fn complete_variants_from_prefix(&self, prefix: &str) -> Vec<String> {
    self
      .variants
      .iter()
      .flatten()
      .filter(|v| match self.case {
        true => v.starts_with(prefix),
        false => v.starts_with(prefix),
      })
      .cloned()
      .collect()
  }
}

//use super::*;
//use regex;
//
//#[derive(Clone)]
//pub struct Ident {
//  variants: Vec<Vec<String>>,
//  def: Option<String>,
//  case: bool,
//}
//
//impl Parsable for Ident {
//  fn can_parse(&self, s: &str) -> bool {
//    self.variants.iter().any(|i| i.iter().any(|i| s == *i))
//  }
//
//  fn complete(&self, s: &str) -> Option<Vec<Completion>> {
//    let res = match s.is_empty() {
//      true => self
//        .variants
//        .iter()
//        .map(|i| Completion::new(0, i[0].clone()).preview(i.join(" | ")))
//        .collect(),
//      false => self
//        .variants
//        .iter()
//        .flatten()
//        .filter_map(|i| {
//          match regex::RegexBuilder::new(&i[..usize::min(i.len(), s.len())])
//            .case_insensitive(!self.case)
//            .build()
//            .unwrap()
//            .find(s)
//          {
//            Some(m) if m.start() == 0 => Some(Completion::new(0, i.clone())),
//            _ => None,
//          }
//        })
//        .collect::<Vec<_>>(),
//    };
//
//    match res.is_empty() {
//      true => None,
//      false => Some(res),
//    }
//  }
//}
//
//impl Ident {
//  pub fn new(variants: &[&[&str]]) -> Self {
//    Self {
//      variants: variants.iter().map(|v| v.iter().map(|s| s.to_string()).collect()).collect(),
//      def: None,
//      case: false,
//    }
//  }
//
//  pub fn no_alternatives(variants: &[&str]) -> Self {
//    Self {
//      variants: variants.iter().map(|v| vec![v.to_string()]).collect(),
//      def: None,
//      case: false,
//    }
//  }
//
//  pub fn default(mut self, def: &str) -> Self {
//    if self.variants.iter().flatten().find(|v| *v == def).is_some() {
//      self.def = Some(def.to_string());
//    }
//    self
//  }
//  pub fn case(mut self, case: bool) -> Self {
//    self.case = case;
//    self
//  }
//}
//
//impl Convert<String> for Ident {
//  fn convert(&self, s: &str) -> Option<String> {
//    self.variants.iter().find(|i| i.iter().any(|i| s == *i)).map(|v| v[0].clone())
//  }
//}
//
//impl ConvertDefault<String> for Ident {
//  fn default(&self) -> Option<String> {
//    self.def.clone()
//  }
//}
