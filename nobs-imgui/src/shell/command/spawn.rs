use super::*;

pub struct Cmd<S: Style, C> {
  cmds: Vec<std::sync::Arc<dyn Command<S, C>>>,
}

impl<S: 'static + Style, C: 'static + Clone + Send> Command<S, C> for Cmd<S, C> {
  fn get_name(&self) -> &'static str {
    "spawn"
  }

  fn get_info(&self) -> (&'static str, &'static str) {
    (
      "run command asynchronously",
      concat!(
        "spawn <cmd> <args..>\nSpawns a new thread that runs the command <cmd> with arguments <args..>\n",
        "Arguments\n",
        "<cmd>      - Name of the command to execute, accepts all commands that are registered with the shell\n",
        "<args...>  - One or more, whitespace separated arguments\n"
      ),
    )
  }

  fn run(&self, args: Vec<String>, shell: Shell<S, C>, context: &mut C) {
    let cmd = if args.len() > 1 {
      args[1..].iter().fold(args[0].clone(), |s, a| format!("{} \"{}\"", s, a))
    } else {
      args[0].clone()
    };

    let name = self.get_name();
    let mut c = context.clone();
    std::thread::spawn(move || {
      shell.exec(&cmd, &mut c);
      shell.get_term().println(&format!("exiting {} {}", name, args[0]))
    });
  }

  fn parse(&self, s: &str) -> Option<Vec<String>> {
    if s.starts_with(self.get_name()) {
      s.chars()
        .skip(self.get_name().len())
        .position(|c| c != ' ')
        .map(|p| &s[p + self.get_name().len()..])
        .and_then(|s| self.cmds.iter().find_map(|c| c.parse(s)))
    } else {
      None
    }
  }
  fn complete(&self, s: &str) -> Option<Vec<arg::Completion>> {
    if self.get_name().starts_with(s) {
      Some(vec![arg::Completion::new(0, self.get_name().into())])
    } else if s.starts_with(self.get_name()) {
      s.chars()
        .skip(self.get_name().len())
        .position(|c| c != ' ')
        .map(|p| &s[p + self.get_name().len()..])
        .and_then(|s| self.cmds.iter().find_map(|c| c.complete(s)))
        .map(|c| c.into_iter().map(|c| c.prefix("spawn ".to_string())).collect())
    } else {
      None
    }
  }
}

impl<S: Style, C> Cmd<S, C> {
  pub fn new(shell: Shell<S, C>) -> Self {
    Self {
      cmds: shell.get_commands(),
    }
  }
}
