use crate::shell::command::args::Arg;
use crate::shell::command::args::ArgDesc;
use crate::shell::command::args::Convert;
use crate::shell::command::args::Matches;

/// Argument to handle a fixed set of identifiers
///
/// Identifiers can have multiple variants, e.g. 'one' and '1', 'two' and '2', ...
#[derive(Clone, Debug)]
pub struct Ident {
  desc: ArgDesc,
  variants: Vec<Vec<String>>,
}

impl Ident {
  /// Creates a new Argument with specified variants
  ///
  /// #Arguments
  ///  * `desc`     - Argument description. In case a default is defined, the default needs to be contained in `variants`
  ///  * `variants` - Identifiers, that are allowed. Strings passed together within the same inner list correspond to the semantically same variant.
  ///                 E.g. [Bool](struct.Bool.html) defines variants as `[[true, 1, on], [false, 0, off]]`.
  ///
  /// #Returns
  /// The Ident argument
  pub fn new(mut desc: ArgDesc, variants: &[&[&str]]) -> Self {
    let make_variant = |v: &[&str]| match v.len() {
      0 => String::new(),
      1 => v[0].to_string(),
      _ => v.iter().skip(1).fold(v[0].to_string(), |acc, s| format!("{} | {}", acc, s)),
    };

    let variants_string = match variants.len() {
      0 => String::new(),
      1 => format!("  [{}]", make_variant(variants[0])),
      _ => variants
        .iter()
        .skip(1)
        .fold(make_variant(variants[0]), |acc, s| format!("{},\n    {}", acc, make_variant(s))),
    };

    desc.help = format!("{}\nPossible values are:\n  [ {} ]", desc.help, variants_string);
    Self {
      desc,
      variants: variants.iter().map(|v| v.iter().map(|s| s.to_string()).collect()).collect(),
    }
  }
}

impl Arg for Ident {
  fn get_desc<'a>(&'a self) -> &'a ArgDesc {
    &self.desc
  }

  fn validate_parsed_value(&self, value: &str) -> bool {
    self.variants.iter().flatten().find(|v| *v == value).is_some()
  }

  fn complete_variants_from_prefix(&self, prefix: &str) -> Vec<String> {
    self.variants.iter().flatten().filter(|v| v.starts_with(prefix)).cloned().collect()
  }
}

impl Convert<String> for Ident {
  fn from_match(&self, matches: &Matches) -> Option<String> {
    matches
      .value_of(&self.get_desc().name)
      .and_then(|name| self.variants.iter().find(|v| v.iter().any(|i| name == *i)).map(|v| v[0].clone()))
  }
}
