use std::ops::Range;

#[derive(Debug)]
pub struct Completion {
  score: i32,
  preview: String,
  completed: String,
  prefix: Option<String>,
  suffix: Option<String>,
}

impl Completion {
  pub fn new(score: i32, completed: String) -> Self {
    Self {
      score,
      preview: completed.clone(),
      completed,
      prefix: None,
      suffix: None,
    }
  }

  pub fn preview(mut self, pv: String) -> Self {
    self.preview = pv;
    self
  }
  pub fn prefix(mut self, pf: String) -> Self {
    self.prefix = self
      .prefix
      .map(|mut s| {
        s.push_str(&pf);
        s
      })
      .or(Some(pf));
    self
  }
  pub fn suffix(mut self, mut sf: String) -> Self {
    self.suffix = self
      .suffix
      .map(|mut s| {
        s.insert_str(0, &sf);
        s
      })
      .or(Some(sf));
    self
  }

  pub fn get_completed(&self) -> String {
    // TODO replace spaces with "\ "
    // only when required (nested call of get_complete in command...)
    format!(
      "{}{}{}",
      match self.prefix.as_ref() {
        Some(pf) => &pf,
        None => "",
      },
      self.completed,
      match self.suffix.as_ref() {
        Some(sf) => &sf,
        None => "",
      },
    )
  }

  pub fn get_preview(&self) -> &str {
    &self.preview
  }
}

impl PartialEq for Completion {
  fn eq(&self, other: &Self) -> bool {
    self.score == other.score && self.completed == other.completed
  }
}

impl Eq for Completion {}

impl PartialOrd for Completion {
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    Some(self.cmp(other))
  }
}

impl Ord for Completion {
  fn cmp(&self, other: &Self) -> std::cmp::Ordering {
    match self.score.cmp(&other.score) {
      std::cmp::Ordering::Equal => self.completed.cmp(&other.completed),
      x => x,
    }
  }
}

pub trait Parsable {
  fn parse(&self, s: &str) -> Option<Vec<String>>;
  fn complete(&self, s: &str) -> Option<Vec<Completion>>;
}

pub struct IdentArg {
  variants: Vec<String>,
}

impl Parsable for IdentArg {
  fn parse(&self, s: &str) -> Option<Vec<String>> {
    self.variants.iter().find(|i| &s == *i).map(|_| vec![s.into()])
  }

  fn complete(&self, s: &str) -> Option<Vec<Completion>> {
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

  fn complete(&self, s: &str) -> Option<Vec<Completion>> {
    use std::path::Path;

    let n = "".to_string();
    let (p, n) = match s.chars().last() {
      Some('/') | Some('\\') => (Path::new(s), n),
      _ => {
        let p = Path::new(s);
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
          let prev = s[p.len() + 1..s.len()].to_string();
          Completion::new(0, s).preview(prev)
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
