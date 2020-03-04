use crate::shell::command::args;
use crate::shell::command::args::Arg;
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

  fn run(&self, matches: &args::Matches, context: &mut C) {
    use std::fs::File;
    use std::io::prelude::*;
    use std::io::BufReader;

    if let Some(file) = self.file.get_match(matches) {
      match File::open(file) {
        Ok(f) => {
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

          println!("=======================");
          for c in cmds.iter() {
            context.println(&c);
            context.get_shell().exec(&c, context);
            println!("{}", c);
          }
        }
        _ => context.println(&format!("Could not open file: {:?}", file)),
      }
    }
  }
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
