use super::*;

pub struct File {}

impl Parsable for File {
  fn can_parse(&self, s: &str) -> bool {
    let p = std::path::Path::new(s);
    if !p.exists() {
      if let Some(p) = p.parent() {
        return p.exists();
      }
    }

    true
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

    if let Ok(dirs) = dir.read_dir() {
      let mut dirs = dirs
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

impl File {
  pub fn new() -> Self {
    Self {}
  }
}
