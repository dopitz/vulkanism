use crate::shell::arg;
use crate::shell::context::Context;
use crate::shell::Command;
use crate::style::Style;

#[derive(Clone)]
pub struct Cmd {
  cmd: arg::Ident,
}

impl<C: Context> Command<C> for Cmd {
  fn get_name(&self) -> &'static str {
    "help"
  }
  fn get_args<'a>(&'a self) -> Vec<&'a dyn arg::Parsable> {
    vec![]
  }
  fn get_opt_args<'a>(&'a self) -> Vec<&'a dyn arg::Parsable> {
    vec![&self.cmd]
  }

  fn get_info(&self) -> (&'static str, &'static str) {
    (
      "prints help",
      "help <cmd name>\nlist commands or description of a single command\nArguments:\n <cmd name> - [Optional] name of the command",
    )
  }

  fn run(&self, args: Vec<String>, context: &mut C) {
    if args.len() == 1 {
      let w = context
        .get_shell()
        .get_commands()
        .iter()
        .fold(0, |w, c| usize::max(w, c.get_name().len()));

      context.println("list of comands:");
      for c in context.get_shell().get_commands().iter() {
        let mut n = c.get_name().to_string();
        while n.len() < w {
          n.push(' ');
        }
        context.println(&format!("  {} -   {}", n, c.get_info().0));
      }
    } else if let Some(cmd) = context.get_shell().get_commands().iter().find(|c| c.get_name() == args[1]) {
      let (short, desc) = cmd.get_info();
      context.println(&format!("{} - {}\n----------------------\n{}", cmd.get_name(), short, desc));
    }
  }
}

impl Cmd {
  pub fn new<C: Context>(cmds: &Vec<std::sync::Arc<dyn Command<C>>>) -> Self {
    let vars = cmds.iter().map(|c| c.get_name().to_string()).collect::<Vec<_>>();
    let cmds = vars.iter().map(|v| v.as_str()).collect::<Vec<_>>();

    Self {
      cmd: arg::Ident::no_alternatives(&cmds),
    }
  }

  pub fn update<C: Context>(&mut self, cmds: &Vec<std::sync::Arc<dyn Command<C>>>) {
    let vars = cmds.iter().map(|c| c.get_name().to_string()).collect::<Vec<_>>();
    let cmds = vars.iter().map(|v| v.as_str()).collect::<Vec<_>>();
    self.cmd = arg::Ident::no_alternatives(&cmds);
  }

  pub fn get_name() -> &'static str {
    "help"
  }
}
