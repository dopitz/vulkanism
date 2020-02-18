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
}
