use super::*;

pub struct Cmd {
  cmd: arg::Ident,
}

impl<S: Style, C> Command<S, C> for Cmd {
  fn get_name(&self) -> &'static str {
    "help"
  }
  fn get_args<'a>(&'a self) -> Vec<&'a arg::Parsable> {
    vec![&self.cmd]
  }

  fn run(&self, args: Vec<String>, shell: Shell<S, C>, _context: &mut C) {
    if let Some(cmd) = shell.get_commands().iter().find(|c| c.get_name() == args[1]) {
      let (short, desc) = cmd.get_info();
      shell
        .get_term()
        .println(&format!("{} - {}\n----------------------\n{}", cmd.get_name(), short, desc));
    }
  }
}

impl Cmd {
  pub fn new<S: Style, C>(cmds: &Vec<std::sync::Arc<dyn Command<S, C>>>) -> Self {
    let vars = cmds.iter().map(|c| c.get_name().to_string()).collect::<Vec<_>>();
    let cmds = vars.iter().map(|v| v.as_str()).collect::<Vec<_>>();

    Self {
      cmd: arg::Ident::no_alternatives(&cmds),
    }
  }

  pub fn get_name() -> &'static str {
    "help"
  }
}
