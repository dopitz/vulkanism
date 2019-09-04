#[derive(Debug)]
pub struct Completion<'a> {
  pub prefix: &'a str,
  pub variant: String,
}

impl<'a> Completion<'a> {
  pub fn new(prefix: &'a str, variant: String) -> Self {
    Self { prefix, variant }
  }

  pub fn completed(&self) -> String {
    format!("{}{}", self.prefix, self.variant)
  }
}

pub trait Parsable {
  fn parse(&self, s: &str) -> Option<Vec<String>>;
  fn complete<'a>(&self, s: &'a str) -> Option<Vec<Completion<'a>>>;
}

pub struct IdentArg {
  variants: Vec<String>,
}

impl Parsable for IdentArg {
  fn parse(&self, s: &str) -> Option<Vec<String>> {
    self.variants.iter().find(|i| &s == *i).map(|_| vec![s.into()])
  }

  fn complete<'a>(&self, s: &'a str) -> Option<Vec<Completion<'a>>> {
    let res = self
      .variants
      .iter()
      .filter(|i| i.starts_with(s))
      .map(|i| Completion::new(&s[0..0], i.to_owned().to_owned()))
      .collect::<Vec<_>>();
    match res.is_empty() {
      true => None,
      false => Some(res),
    }
  }
}

impl IdentArg {
  pub fn new(variants: Vec<String>) -> Self {
    Self { variants }
  }

  pub fn from_slice(variants: &[&str]) -> Self {
    Self {
      variants: variants.iter().map(|v| v.to_string()).collect(),
    }
  }
}

pub struct FileArg {}

impl Parsable for FileArg {
  fn parse(&self, s: &str) -> Option<Vec<String>> {
    // TODO: check if s is a valid path (non existing files/folders?)
    Some(vec![s.to_string()])
  }

  fn complete<'a>(&self, s: &'a str) -> Option<Vec<Completion<'a>>> {
    if let Ok(dirs) = std::fs::read_dir(s) {
      let mut c = Vec::new();
      for fname in dirs.filter_map(|d| d.ok().and_then(|d| d.file_name().into_string().ok())) {
        c.push(Completion::new(&s[0..0], format!("/{}", fname)));
      }
      Some(c)
    } else {
      None
    }
  }
}

impl FileArg {
  pub fn new() -> Self {
    Self {}
  }
}
