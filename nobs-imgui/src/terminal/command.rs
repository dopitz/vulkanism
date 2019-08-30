use super::Terminal;

pub struct Command<'a, R: Runnable, S: Style> {
  runnable: R,
  term: Terminal<S>,
}


pub trait Runnable {
  fn name(&'a self) -> &'a str;

  fn run();
}
