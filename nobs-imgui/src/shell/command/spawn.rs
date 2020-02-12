use crate::shell::arg;
use crate::shell::context::Context;
use crate::shell::Command;
use crate::style::Style;

pub struct Cmd<C: Context> {
  cmds: Vec<std::sync::Arc<dyn Command<Context = C>>>,
}

impl<C: 'static + Clone + Send + Context<ShellContext = C>> Command for Cmd<C> {
  type Context = C;

  fn get_name(&self) -> &'static str {
    "spawn"
  }

  fn get_info(&self) -> (&'static str, &'static str) {
    (
      "run command asynchronously",
      concat!(
        "spawn <cmd> <args..> [--- <cmd> <args..> ..]\n",
        "Spawns a new thread that runs the command <cmd> with arguments <args..>\n",
        "Multiple commands separated with '---' may be passed to the new thread and executed in sequence.\n\n",
        "Arguments\n",
        "<cmd>     - Name of the command to execute, accepts all commands that are registered with the shell\n",
        "<args..>  - One or more, whitespace separated arguments\n\n",
        "Example\n",
        "spawn cmd1 --- cmd2 a b c --- cmd3 x y\n",
        "This will spawn a new thread that executes the three commands in sequence\n"
      ),
    )
  }

  fn run(&self, args: Vec<String>, context: &mut C) {
    let mut c = context.clone();
    std::thread::spawn(move || {
      for a in args.iter().skip(1) {
        c.println(&a);
        c.get_shell().exec(&a, &mut c);
      }
      c.println("exiting");
    });
  }

  fn parse(&self, s: &str) -> Option<Vec<String>> {
    if s.starts_with(self.get_name()) {
      let s = s[self.get_name().len()..]
        .split("---")
        .map(|s| s.trim().to_string())
        .filter(|s| self.cmds.iter().find(|c| c.parse(&s).is_some()).is_some())
        .fold(vec![self.get_name().to_string()], |mut acc, arg| {
          acc.push(arg);
          acc
        });
      if s.is_empty() {
        None
      } else {
        Some(s)
      }
    } else {
      None
    }
  }
  fn complete(&self, s: &str) -> Option<Vec<arg::Completion>> {
    if self.get_name().starts_with(s) {
      Some(vec![arg::Completion::new(0, self.get_name().into())])
    } else if s.starts_with(self.get_name()) {
      let len = s.rfind("---").map(|p| p + 3).unwrap_or(self.get_name().len());
      s.chars()
        .skip(len)
        .position(|c| c != ' ')
        .or_else(|| if s.len() > len { Some(s.len() - len) } else { None })
        .map(|p| (&s[..p + len], &s[p + len..]))
        .map(|(prefix, s)| {
          self
            .cmds
            .iter()
            .filter_map(|c| c.complete(s))
            .flatten()
            .map(|c| c.prefix(prefix.to_string()))
            .collect::<Vec<_>>()
        })
    } else {
      None
    }
  }
}

impl<C: Context> Cmd<C> {
  pub fn new(cmds: Vec<std::sync::Arc<dyn Command<Context = C>>>) -> Self {
    Self { cmds }
  }
}
