use crate::shell::command::args::Arg;
use crate::shell::command::args::ArgDesc;
use crate::shell::command::args::Convert;
use crate::shell::command::args::Matches;

/// Argument to handle free text parameters
///
/// The argument value may be any string. If the string contains whitespaces, the argument must be passed withn double quotes.
#[derive(Clone, Debug)]
pub struct Text {
  desc: ArgDesc,
}

impl Text {
  /// Creates a new Argument
  ///
  /// #Arguments
  ///  * `desc`     - Argument description.
  ///
  /// #Returns
  /// The Text argument
  pub fn new(mut desc: ArgDesc) -> Self {
    Self { desc }
  }
}

impl Arg for Text {
  fn get_desc<'a>(&'a self) -> &'a ArgDesc {
    &self.desc
  }
}

impl Convert<String> for Text {
  fn from_match(&self, matches: &Matches) -> Option<String> {
    self.get_match(matches).map(|s| s.to_string())
  }
}
