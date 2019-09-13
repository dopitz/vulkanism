use super::*;
use regex;

pub struct Ident {
  variants: Vec<Vec<String>>,
  case: bool,
}

impl Parsable for Ident {
  fn can_parse(&self, s: &str) -> bool {
    self.variants.iter().any(|i| i.iter().any(|i| s == *i))
  }

  fn complete(&self, s: &str) -> Option<Vec<Completion>> {
    let res = match s.is_empty() {
      true => self
        .variants
        .iter()
        .map(|i| Completion::new(0, i[0].clone()).preview(i.join(" | ")))
        .collect(),
      false => self
        .variants
        .iter()
        .flatten()
        .filter_map(|i| {
          match regex::RegexBuilder::new(&i[..usize::min(i.len(), s.len())])
            .case_insensitive(!self.case)
            .build()
            .unwrap()
            .find(s)
          {
            Some(m) if m.start() == 0 => Some(Completion::new(0, i.clone())),
            _ => None,
          }
        })
        .collect::<Vec<_>>(),
    };

    match res.is_empty() {
      true => None,
      false => Some(res),
    }
  }
}

impl Ident {
  pub fn new(variants: &[&[&str]]) -> Self {
    Self {
      variants: variants.iter().map(|v| v.iter().map(|s| s.to_string()).collect()).collect(),
      case: false,
    }
  }

  pub fn no_alternatives(variants: &[&str]) -> Self {
    Self {
      variants: variants.iter().map(|v| vec![v.to_string()]).collect(),
      case: false,
    }
  }

  pub fn case(mut self, case: bool) -> Self {
    self.case = case;
    self
  }

  pub fn convert<'a>(&'a self, s: &str) -> Option<&'a str> {
    self.variants.iter().find(|i| i.iter().any(|i| s == *i)).map(|v| v[0].as_str())
  }
}
