use super::Shell;
use super::Terminal;
use crate::style::Style;

pub trait Context {
  type TerminalStyle: Style;
  type ShellContext: Context;

  fn get_shell(&self) -> &Shell<Self::ShellContext>;
  fn get_term(&self) -> &Terminal<Self::TerminalStyle>;

  fn print(&self, s: &str) {
    self.get_term().print(s);
  }
  fn println(&self, s: &str) {
    self.get_term().println(s);
  }
  fn readln(&self) -> String {
    self.get_term().readln()
  }
}
