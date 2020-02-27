use crate::shell::command::args::Arg;
use crate::shell::command::args::ArgDesc;

/// Special Argument specifying the command name at `index == 0`
#[derive(Clone, Debug)]
pub struct File {
  desc: ArgDesc,
  ext: Option<String>,
}

impl File {
  pub fn new(desc: ArgDesc) -> Self {
    Self { desc, ext: None }
  }

  pub fn with_extension(desc: ArgDesc, ext: &str) -> Self {
    Self {
      desc,
      ext: Some(ext.to_string()),
    }
  }
}

impl Arg for File {
  fn get_desc<'a>(&'a self) -> &'a ArgDesc {
    &self.desc
  }

  fn complete_variants_from_prefix(&self, prefix: &str) -> Vec<String> {
    use std::path::Path;

    let filename_prefix = "".to_string();
    let (dirname, filename_prefix) = match prefix.chars().last() {
      Some('/') | Some('\\') => (Path::new(prefix), filename_prefix),
      _ => {
        let p = Path::new(prefix);
        let filename_prefix = match p.file_name().and_then(|p| p.to_os_string().into_string().ok()) {
          Some(n) => n,
          None => filename_prefix,
        };

        match p.parent() {
          Some(p) => match p.to_str() {
            Some("") => (Path::new("./"), filename_prefix),
            _ => (p, filename_prefix),
          },
          None => (Path::new("./"), filename_prefix),
        }
      }
    };

    if let Ok(ls) = dirname.read_dir() {
      let mut ls = ls
        .filter_map(|d| d.ok().and_then(|d| d.file_name().into_string().ok()))
        .filter(|fname| fname.starts_with(&filename_prefix))
        .map(|fname| {
          let p = match dirname.to_str() {
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
          s.to_string()
          //s[p.len()..s.len()].to_string()
          //Completion::new(0, s).preview(prev)
        })
        //.filter(|p| {
        //  let p = &p.get_completed();
        //  let is_file = Path::new(&p).is_file();
        //  match self.ext.as_ref() {
        //    Some(ext) if is_file => p.ends_with(ext),
        //    _ => true,
        //  }
        //})
        .collect::<Vec<_>>();

      ls.sort();
      ls
    } else {
      vec![]
    }
  }
}
