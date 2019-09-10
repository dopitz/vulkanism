use super::*;

pub struct Bool {
  ident: Ident,
}

impl Parsable for Bool {
  fn parse(&self, s: &str) -> Option<Vec<String>> {
    self.ident.parse(s)
  }

  fn complete(&self, s: &str) -> Option<Vec<Completion>> {
    self.ident.complete(s)
  }
}

impl Bool {
  pub fn new() -> Self {
    Self {
      ident: Ident::new(&[&["On", "true", "1"], &["Off", "false", "0"]]),
    }
  }

  fn convert(&self, s: &str) -> Option<bool> {
    self.ident.convert(s).map(|b| b == "On")
  }
}
