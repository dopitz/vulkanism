use super::*;

#[derive(Clone)]
pub struct Cmd {
  cmd: arg::Ident,
}

impl<S: Style, C> Command<S, C> for Cmd {
  fn get_name(&self) -> &'static str {
    "help"
  }
  fn get_args<'a>(&'a self) -> Vec<&'a arg::Parsable> {
    vec![]
  }
  fn get_opt_args<'a>(&'a self) -> Vec<&'a arg::Parsable> {
    vec![&self.cmd]
  }

  fn get_info(&self) -> (&'static str, &'static str) {
    (
      "prints help",
      "help <cmd name>\nlist commands or description of a single command\nArguments:\n <cmd name> - [Optional] name of the command",
    )
  }

  fn run(&self, args: Vec<String>, term: Terminal<S, C>, _context: &mut C) {
    if args.len() == 1 {
      let w = term.shell.get_commands().iter().fold(0, |w, c| usize::max(w, c.get_name().len()));

      term.println("list of commands:");
      for c in term.shell.get_commands().iter() {
        let mut n = c.get_name().to_string();
        while n.len() < w {
          n.push(' ');
        }
        term.println(&format!("  {} -   {}", n, c.get_info().0));
      }
    } else if let Some(cmd) = term.shell.get_commands().iter().find(|c| c.get_name() == args[1]) {
      let (short, desc) = cmd.get_info();
      term.println(&format!("{} - {}\n----------------------\n{}", cmd.get_name(), short, desc));
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

  pub fn update<S: Style, C>(&mut self, cmds: &Vec<std::sync::Arc<dyn Command<S, C>>>) {
    let vars = cmds.iter().map(|c| c.get_name().to_string()).collect::<Vec<_>>();
    let cmds = vars.iter().map(|v| v.as_str()).collect::<Vec<_>>();
    self.cmd = arg::Ident::no_alternatives(&cmds);
  }

  pub fn get_name() -> &'static str {
    "help"
  }
}
