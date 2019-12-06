use super::*;

pub struct File {
  ext: Option<String>,
  def: Option<String>,
}

impl Parsable for File {
  fn can_parse(&self, s: &str) -> bool {
    let p = std::path::Path::new(s);
    if !p.exists() {
      match p.parent() {
        Some(p) => p.exists() || p.to_str().unwrap() == "",
        _ => false,
      }
    } else if let Some(ext) = self.ext.as_ref() {
      s.ends_with(ext)
    } else {
      true
    }
  }

  fn complete(&self, s: &str) -> Option<Vec<Completion>> {
    use std::path::Path;

    let n = "".to_string();
    let (dir, name) = match s.chars().last() {
      Some('/') | Some('\\') => (Path::new(s), n),
      _ => {
        let p = Path::new(s);
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

    if let Ok(ls) = dir.read_dir() {
      let mut ls = ls
        .filter_map(|d| d.ok().and_then(|d| d.file_name().into_string().ok()))
        .filter(|fname| fname.starts_with(&name))
        .map(|fname| {
          let p = match dir.to_str() {
            Some(p) => p,
            None => "",
          };
          let s = format!(
            "{}{}{}",
            p,
            match p.chars().last() {
              Some('/') | Some('\\') => "",
              _ => "/",
            },
            fname
          );
          let prev = s[p.len()..s.len()].to_string();
          Completion::new(0, s).preview(prev)
        })
        .filter(|p| {
          let p = &p.get_completed();
          let is_file = Path::new(&p).is_file();
          match self.ext.as_ref() {
            Some(ext) if is_file => p.ends_with(ext),
            _ => true,
          }
        })
        .collect::<Vec<_>>();

      ls.sort();

      match ls.is_empty() {
        true => None,
        false => Some(ls),
      }
    } else {
      None
    }
  }
}

impl File {
  pub fn new() -> Self {
    Self { ext: None, def: None }
  }

  pub fn ext(mut self, ext: &str) -> Self {
    self.ext = Some(ext.into());
    self
  }
  pub fn default(mut self, def: &str) -> Self {
    self.def = Some(def.into());
    self
  }
}

impl Convert<String> for File {
  fn convert(&self, s: &str) -> Option<String> {
    Some(s.to_string())
  }
}

impl ConvertDefault<String> for File {
  fn default(&self) -> Option<String> {
    self.def.clone()
  }
}
