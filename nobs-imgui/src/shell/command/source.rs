use crate::shell::arg;
use crate::shell::context::Context;
use crate::shell::Command;
use crate::style::Style;

pub struct Cmd {
  file: arg::File,
}

impl<C: Context> Command<C> for Cmd {
  fn get_name(&self) -> &'static str {
    "source"
  }
  fn get_args<'a>(&'a self) -> Vec<&'a dyn arg::Parsable> {
    vec![&self.file]
  }

  fn get_info(&self) -> (&'static str, &'static str) {
    (
      "run commands from file",
      concat!(
        "source <file>\n",
        "Runs commands from input file\n",
        "Commands in the file are interpreted line wise. Use '\\' at the end of a line to escape the newline\n",
        "Example file:\n",
        "  cmd1\n",
        "  cmd2 a b\n",
        "  cmd3 multi\\\n",
        "  line command\n",
        "  cmd1\n",
        "Note that commands used in the file must be defined for the shell, otherwise they are ignored and skipped\n\n",
        "Arguments\n",
        "<file>  - Path to the file with the commands\n",
      ),
    )
  }

  fn run(&self, args: Vec<String>, context: &mut C) {
    use std::fs::File;
    use std::io::prelude::*;
    use std::io::BufReader;

    if let Ok(f) = File::open(&args[1]) {
      let mut cmds: Vec<String> = Vec::new();
      let mut escape = false;
      let reader = BufReader::new(f);

      let mut push = |s: String, escape: bool| {
        if escape {
          if let Some(c) = cmds.last_mut().as_mut() {
            c.push_str(&s);
          }
        } else {
          cmds.push(s);
        }
      };

      for l in reader.lines().filter_map(|l| l.ok()) {
        let l = l.trim();
        if let Some('\\') = l.chars().rev().next() {
          push(l.replace("\\", " "), escape);
          escape = true;
        } else {
          push(l.to_string(), escape);
          escape = false;
        }
      }

      for c in cmds.iter() {
        context.println(&c);
        if let Some(exe) = context.get_shell().parse(&c) {
          exe.run(context);
        }
      }
    } else {
      context.println(&format!("Could not open file: {:?}", args[1]));
    }
  }
}

impl Cmd {
  pub fn new() -> Self {
    Self { file: arg::File::new() }
  }
}
