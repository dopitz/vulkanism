use crate::shell::command::args::Arg;
use crate::shell::command::args::ArgDesc;

/// Special Argument specifying the command name at `index == 0`
#[derive(Clone, Debug)]
pub struct CommandName {
  desc: ArgDesc,
}

impl CommandName {
  pub fn new(name: &str, help: &str) -> Self {
    Self {
      desc: ArgDesc::new(name).index(0).help(help),
    }
  }
}

impl Arg for CommandName {
  fn get_desc<'a>(&'a self) -> &'a ArgDesc {
    &self.desc
  }
}
