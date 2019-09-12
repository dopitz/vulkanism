use super::*;

pub struct Cmd {
  file: arg::File,
}

impl<S: Style, C> Command<S, C> for Cmd {
  fn get_name(&self) -> &'static str {
    "source"
  }
  fn get_args<'a>(&'a self) -> Vec<&'a arg::Parsable> {
    vec![&self.file]
  }

  fn get_info(&self) -> (&'static str, &'static str) {
    (
      "run commands from file",
      "Reads file linewise, where each line will be interpreted as a command input.
Note that commands used in the file must be defined for the shell, otherwise they are ignored and skipped",
    )
  }

  fn run(&self, args: Vec<String>, shell: Shell<S, C>, context: &mut C) {
    let term = shell.get_term();

    use std::fs::File;
    use std::io::prelude::*;
    use std::io::BufReader;

    if let Ok(f) = File::open(&args[1]) {
      let reader = BufReader::new(f);
      for l in reader.lines().filter_map(|l| l.ok()) {
        term.println(&l);
        shell.exec(&l, context);
      }
    } else {
      term.println(&format!("Could not open file: {:?}", args[1]));
    }
  }
}

impl Cmd {
  pub fn new() -> Self {
    Self { file: arg::File::new() }
  }
}
