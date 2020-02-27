use crate::shell::command::args::Arg;
use crate::shell::command::args::ArgDesc;
use crate::shell::command::args::Convert;
use crate::shell::command::args::Ident;
use crate::shell::command::args::Matches;

/// Argument to handle a fixed set of identifiers
///
/// Identifiers can have multiple variants, e.g. 'one' and '1', 'two' and '2', ...
#[derive(Clone, Debug)]
pub struct Bool {
  ident: Ident,
}

impl Bool {
  /// Creates a new Argument with specified variants
  ///
  /// #Arguments
  ///  * `desc`     - Argument description. In case a default is defined, the default needs to be contained in `variants`
  ///  * `variants` - Identifiers, that are allowed. Strings passed together within the same inner list correspond to the semantically same variant.
  ///                 E.g. [Bool](struct.Bool.html) defines variants as `[[true, 1, on], [false, 0, off]]`.
  ///
  /// #Returns
  /// The Ident argument
  pub fn new(desc: ArgDesc) -> Self {
    Self {
      ident: Ident::new(desc, &[&["on", "true", "1"], &["off", "false", "0"]]),
    }
  }
}

impl Arg for Bool {
  fn get_desc<'a>(&'a self) -> &'a ArgDesc {
    self.ident.get_desc()
  }

  fn complete_variants_from_prefix(&self, prefix: &str) -> Vec<String> {
    self.ident.complete_variants_from_prefix(prefix)
  }
}

impl Convert<bool> for Bool {
  fn from_match(&self, matches: &Matches) -> Option<bool> {
    self.ident.from_match(matches).map(|v| v == "on")
  }
}
