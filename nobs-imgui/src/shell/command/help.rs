use crate::shell::command::args;
use crate::shell::command::args::Matches;
use crate::shell::command::Command;
use crate::shell::context::ContextShell;

#[derive(Clone)]
pub struct Cmd {
  thisname: args::CommandName,
  cmd: args::Ident,
}

impl<C: ContextShell> Command<C> for Cmd {
  fn get_args<'a>(&'a self) -> Vec<&'a dyn args::Arg> {
    vec![&self.thisname, &self.cmd]
  }

  fn run(&self, matches: &Matches, context: &mut C) -> Result<(), String> {
    match matches.value_of("command") {
      Some(cmd) => match context.get_shell().get_commands().iter().find(|c| c.get_commandname() == cmd) {
        Some(cmd) => context.println(&cmd.get_help()),
        None => context.println(&format!("unknown command: {}", cmd)),
      },
      None => {
        context.println("list of commands:");

        let cmds = context.get_shell().get_commands();
        let args = cmds.iter().map(|c| c.get_args()).collect::<Vec<_>>();
        let descs = args
          .iter()
          .map(|a| a.iter().find(|a| a.get_desc().index.filter(|i| *i == 0).is_some()).unwrap())
          .map(|a| a.get_desc())
          .collect::<Vec<_>>();

        context.println(&args::ArgDesc::format_help(&descs, false));
      }
    }
    Ok(())
  }
}

impl Cmd {
  pub fn new<C: ContextShell>(cmds: &Vec<std::sync::Arc<dyn Command<C>>>) -> Self {
    let vars = cmds.iter().map(|c| c.get_commandname().to_string()).collect::<Vec<_>>();
    let cmds = vars.iter().map(|v| v.as_str()).collect::<Vec<_>>();

    Self {
      thisname: args::CommandName::new("help", "Lists available commands and prints usage information."),
      cmd: args::Ident::new(
        args::ArgDesc::new("command")
          .index(1)
          .optional(true)
          .help("Command name for which usage information should be shown."),
        &[cmds.as_slice()],
      ),
    }
  }

  pub fn update<C: ContextShell>(&mut self, cmds: &Vec<std::sync::Arc<dyn Command<C>>>) {
    let vars = cmds.iter().map(|c| c.get_commandname().to_string()).collect::<Vec<_>>();
    let cmds = vars.iter().map(|v| v.as_str()).collect::<Vec<_>>();
    self.cmd = args::Ident::new(
      args::ArgDesc::new("command")
        .index(1)
        .optional(true)
        .help("Command name for which usage information should be shown."),
      &[cmds.as_slice()],
    );
  }

  pub fn get_name() -> &'static str {
    "help"
  }
}
