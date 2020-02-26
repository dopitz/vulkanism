use crate::shell::command::args;
use crate::shell::command::args::Parsed;
use crate::shell::context::Context;
use crate::shell::Command;

pub struct Cmd {
  thisname: args::CommandName,
  file: args::File,
}

impl<C: Context> Command<C> for Cmd {
  fn get_args<'a>(&'a self) -> Vec<&'a dyn args::Arg> {
    vec![&self.thisname, &self.file]
  }

  fn run(&self, args: &[Parsed], context: &mut C) {
    use std::fs::File;
    use std::io::prelude::*;
    use std::io::BufReader;

    //if let Ok(f) = File::open(&args[1]) {
    //  let mut cmds: Vec<String> = Vec::new();
    //  let mut escape = false;
    //  let reader = BufReader::new(f);

    //  let mut push = |s: String, escape: bool| {
    //    if escape {
    //      if let Some(c) = cmds.last_mut().as_mut() {
    //        c.push_str(&s);
    //      }
    //    } else {
    //      cmds.push(s);
    //    }
    //  };

    //  for l in reader.lines().filter_map(|l| l.ok()) {
    //    let l = l.trim();
    //    if let Some('\\') = l.chars().rev().next() {
    //      push(l.replace("\\", " "), escape);
    //      escape = true;
    //    } else {
    //      push(l.to_string(), escape);
    //      escape = false;
    //    }
    //  }

    //  for c in cmds.iter() {
    //    context.println(&c);
    //    context.get_shell().exec(&c, context);
    //  }
    //} else {
    //  context.println(&format!("Could not open file: {:?}", args[1]));
    //}
  }

  //fn get_info(&self) -> (&'static str, &'static str) {
  //  (
  //    "run commands from file",
  //    concat!(
  //      "source <file>\n",
  //      "Runs commands from input file\n",
  //      "Commands in the file are interpreted line wise. Use '\\' at the end of a line to escape the newline\n",
  //      "Example file:\n",
  //      "  cmd1\n",
  //      "  cmd2 a b\n",
  //      "  cmd3 multi\\\n",
  //      "  line command\n",
  //      "  cmd1\n",
  //      "Note that commands used in the file must be defined for the shell, otherwise they are ignored and skipped\n\n",
  //      "Arguments\n",
  //      "<file>  - Path to the file with the commands\n",
  //    ),
  //  )
  //}

  //fn run(&self, args: Vec<String>, context: &mut C) {
  //  use std::fs::File;
  //  use std::io::prelude::*;
  //  use std::io::BufReader;

  //  if let Ok(f) = File::open(&args[1]) {
  //    let mut cmds: Vec<String> = Vec::new();
  //    let mut escape = false;
  //    let reader = BufReader::new(f);

  //    let mut push = |s: String, escape: bool| {
  //      if escape {
  //        if let Some(c) = cmds.last_mut().as_mut() {
  //          c.push_str(&s);
  //        }
  //      } else {
  //        cmds.push(s);
  //      }
  //    };

  //    for l in reader.lines().filter_map(|l| l.ok()) {
  //      let l = l.trim();
  //      if let Some('\\') = l.chars().rev().next() {
  //        push(l.replace("\\", " "), escape);
  //        escape = true;
  //      } else {
  //        push(l.to_string(), escape);
  //        escape = false;
  //      }
  //    }

  //    for c in cmds.iter() {
  //      context.println(&c);
  //      context.get_shell().exec(&c, context);
  //    }
  //  } else {
  //    context.println(&format!("Could not open file: {:?}", args[1]));
  //  }
  //}
}

impl Cmd {
  pub fn new() -> Self {
    Self {
      thisname: args::CommandName::new(
        "source",
        concat!(
          "Sources a .aoeu file with commands.\n",
          "All commands in the sourced file need to be supported by the runtime shell.\n",
          "If a command is unknown or could not be parsed it will be skipped."
        ),
      ),
      file: args::File::new(
        args::ArgDesc::new("file")
          .index(1)
          .short("f")
          .help("Path to the file that is to be sourced."),
      ),
    }
  }
}
