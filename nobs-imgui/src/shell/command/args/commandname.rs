use crate::shell::command::args::Arg;
use crate::shell::command::args::ArgDesc;

/// Special Argument specifying the command name at `index == 0`
pub struct CommandName {
  desc: ArgDesc,
}

impl CommandName {
  pub fn new(name: &str) -> Self {
    Self {
      desc: ArgDesc::new(name).index(0),
    }
  }
}

impl Arg for CommandName {
  fn get_desc<'a>(&'a self) -> &'a ArgDesc {
    &self.desc
  }
}
