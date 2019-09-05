use std::ops::Range;

#[derive(Debug)]
pub struct Completion<'a> {
  score: i32,
  completed: String,
  preview: Range<usize>,
  prefix: Option<&'a str>,
  suffix: Option<&'a str>,
}

impl<'a> Completion<'a> {
  pub fn new(score: i32, completed: String) -> Self {
    Self {
      score,
      preview: 0..completed.len(),
      completed,
      prefix: None,
      suffix: None,
    }
  }

  pub fn preview(mut self, pv: Range<usize>) -> Self {
    self.preview = pv;
    self
  }
  pub fn prefix(mut self, pf: &'a str) -> Self {
    self.prefix = Some(pf);
    self
  }
  pub fn suffix(mut self, sf: &'a str) -> Self {
    self.suffix = Some(sf);
    self
  }

  pub fn get_completed(&self) -> String {
    // TODO replace spaces with "\ "
    // only when required (nested call of get_complete in command...)
    format!(
      "{}{}{}",
      match self.prefix {
        Some(pf) => pf,
        None => "",
      },
      self.completed,
      match self.suffix {
        Some(sf) => sf,
        None => "",
      },
    )
  }

  pub fn get_preview(&self) -> &str {
    &self.completed[self.preview.clone()]
  }
}

impl<'a> PartialEq for Completion<'a> {
  fn eq(&self, other: &Self) -> bool {
    self.score == other.score && self.completed == other.completed
  }
}

impl<'a> Eq for Completion<'a> {}

impl<'a> PartialOrd for Completion<'a> {
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    Some(self.cmp(other))
  }
}

impl<'a> Ord for Completion<'a> {
  fn cmp(&self, other: &Self) -> std::cmp::Ordering {
    match self.score.cmp(&other.score) {
      std::cmp::Ordering::Equal => self.completed.cmp(&other.completed),
      x => x,
    }
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
      .map(|i| Completion::new(0, i.to_string()))
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
            Some("") => (Path::new("."), n),
            _ => (p, n),
          },
          None => (Path::new("."), n),
        }
      }
    };

    if let Ok(dirs) = p.read_dir() {
      let mut dirs = dirs
        .filter_map(|d| d.ok().and_then(|d| d.file_name().into_string().ok()))
        .filter(|fname| fname.starts_with(&n))
        .map(|fname| {
          let p = match p.to_str() {
            Some(p) => p,
            None => "",
          };
          let s = format!("{}/{}", p, fname);
          let r = p.len() + 1..s.len();
          Completion::new(0, s).preview(r)
        })
        .collect::<Vec<_>>();

      dirs.sort();

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
