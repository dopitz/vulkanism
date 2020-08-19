use super::matches;

use crate::shell::command::args::Arg;
use crate::shell::command::args::ArgDesc;
use crate::shell::command::args::Convert;
use crate::shell::command::args::Matches;

/// Argument to handle a fixed set of identifiers
///
/// Identifiers can have multiple variants, e.g. 'one' and '1', 'two' and '2', ...
#[derive(Clone, Debug)]
pub struct Flag {
  desc: ArgDesc,
}

impl Flag {
  /// Creates a new Argument with specified variants
  ///
  /// #Arguments
  ///  * `desc`     - Argument description. In case a default is defined, the default needs to be contained in `variants`
  ///  * `variants` - Identifiers, that are allowed. Strings passed together within the same inner list correspond to the semantically same variant.
  ///                 E.g. [Bool](struct.Bool.html) defines variants as `[[true, 1, on], [false, 0, off]]`.
  ///
  /// #Returns
  /// The Ident argument
  pub fn new(mut desc: ArgDesc) -> Self {
    Self { desc }
  }
}

impl Arg for Flag {
  fn get_desc<'a>(&'a self) -> &'a ArgDesc {
    &self.desc
  }

  fn parse<'a>(
    &'a self,
    argsv: &'a [&'a str],
    argi: usize,
    completions: &mut Option<Vec<matches::Completion>>,
  ) -> Option<(usize, matches::Parsed)> {
    // no more argument strings to consume
    if argi >= argsv.len() {
      return None;
    }

    if let Some(argivalue) = self.parse_name(argsv, argi, completions) {
      Some((
        argivalue,
        matches::Parsed {
          name: self.get_desc().name.clone(),
          value: "".to_string(),
        },
      ))
    } else {
      None
    }
  }
}
