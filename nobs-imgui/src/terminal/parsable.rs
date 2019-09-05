use std::ops::Range;

#[derive(Debug)]
pub struct Completion<'a> {
  score: i32,
  completed: String,
  preview: Range<usize>,
  prefix: &'a str,
  suffix: &'a str,
}

impl<'a> Completion<'a> {
  pub fn new(score: i32, prefix: &'a str, completed: String, suffix: &'a str, preview: Range<usize>) -> Self {
    Self {
      score,
      completed,
      preview,
      prefix,
      suffix,
    }
  }

  pub fn completed(&self) -> String {
    format!("{}{}{}", self.prefix, self.completed, self.suffix)
  }

  pub fn preview(&self) -> &str {
    &self.completed[self.preview.clone()]
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
      .map(|i| Completion::new(0, &s[0..0], i.to_string(), &s[0..0], Range { start: 0, end: i.len() }))
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
    use std::path::Path;

    let n = "".to_string();
    let (mut p, n) = match s.chars().last() {
      Some('/') | Some('\\') => (Path::new(s), n),
      _ => {
        let mut p = Path::new(s);
        let n = match p.file_name().and_then(|p| p.to_os_string().into_string().ok()) {
          Some(n) => n,
          None => n,
        };

        match p.parent() {
          Some(p) => match p.to_str() {
            Some("") => (Path::new("./"), n),
            _ => (p, n),
          },
          None => (Path::new("./"), n),
        }
      }
    };

    if let Ok(dirs) = p.read_dir() {
      let mut dirs = dirs
        .filter_map(|d| d.ok().and_then(|d| d.file_name().into_string().ok()))
        .filter(|fname| fname.starts_with(&n))
        .map(|fname| {
          Completion::new(
            0,
            &s[0..0],
            format!("{}", fname),
            &s[0..0],
            Range {
              start: 0,
              end: fname.len(),
            },
          )
        })
        .collect::<Vec<_>>();

      match dirs.is_empty() {
        true => None,
        false => Some(dirs),
      }
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
