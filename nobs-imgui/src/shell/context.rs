use crate::shell::Shell;
use crate::shell::Terminal;
use crate::style::Style;

pub trait ContextBase: std::marker::Sized {
  fn print(&self, s: &str);
  fn println(&self, s: &str);
  fn readln(&self) -> String;
}

pub trait ContextShell: std::marker::Sized + ContextBase {
  type TerminalStyle: Style;

  fn get_shell(&self) -> Shell<Self>;
  fn get_term(&self) -> Terminal<Self::TerminalStyle>;
}

pub struct Standalone {}

impl Standalone {
  pub fn new() -> Self {
    Standalone {}
  }
}

impl ContextBase for Standalone {
  fn print(&self, s: &str) {
    print!("{}", s);
  }

  fn println(&self, s: &str) {
    println!("{}", s);
  }

  fn readln(&self) -> String {
    return String::new();
  }
}
