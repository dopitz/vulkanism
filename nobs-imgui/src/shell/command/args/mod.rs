/// Argument desrciption
///
/// Lists properties of a command line argument.
///
/// Arguments must be uniqely identified by a `name` and/or index.
/// The argument `name` must always be specified.
/// Argument `index` starts counting from 1, `index == 0` is reserved for [CommandName](struct.CommandName.html).
/// Multiple arguments of the same [Command](../trait.Command.html) with the same `name` or `index` are illeagal.
/// If `short` is specified, it must not clash with the `short` identifier of other arguments in the same [Command](../trait.Command.html).
///
/// Impletents a builder pattern for it's members
#[derive(Clone, Debug)]
pub struct ArgDesc {
  /// Index of the argument
  ///
  /// - A unique `index` may be specified for any argument
  /// - The argument value can then be specified without preceeding `name` or `short`
  /// - The indices must start with `index == 1` and in sequence without gaps.
  /// - `index == 0` is reserved for [CommandName](struct.CommandName.html)
  /// - By default no index is used
  pub index: Option<usize>,
  /// Name of the argument
  ///
  /// - A unique `name` must be specified for all arguments
  /// - The argument value can then be specified with "--<`name`> <value>".
  ///   If value contains whitespaces, they may be escaped with '\' or the value string may be enclosed in double quotes (")
  pub name: String,
  /// Short name of the argument
  ///
  /// - A unique `short` may be specified for any argument
  /// - The argument value can then be specified with "-<`short`> <value>".
  pub short: Option<String>,
  /// Default value for the argument
  ///
  /// - May be specidied for any argument
  /// - In case no value was parsed for this argument it is automatically assigned to the specifid `default`
  pub default: Option<String>,
  /// Help text for the default value of this argument
  ///
  /// - May be specidied for any argument
  /// - Sometimes the default value depends on another argument (e.g. same as value of argument X with different file extension).
  ///   In this case [format_help](struct.ArgDesc.html#method.format_help)
  pub default_help: Option<String>,
  /// Declares the argument as optional
  ///
  /// - May be specified for any argument
  /// - By default all arguments are manditory
  /// - If `optional == true` parsing will not fail if no value was specified for this argument
  pub optional: bool,
  /// Information for this arguments usage
  ///
  /// - May be specified for any argument
  /// - Used to generate user manual
  pub help: String,
}

impl ArgDesc {
  /// Create now argument description with 'name'
  ///
  /// #Arguments
  /// * `name` - [Name](ArgDesc.html#members.name) of the parameter
  pub fn new(name: &str) -> Self {
    Self {
      index: None,
      name: name.to_string(),
      short: Default::default(),
      default: None,
      default_help: None,
      optional: false,
      help: Default::default(),
    }
  }

  pub fn index(mut self, i: usize) -> Self {
    self.index = Some(i);
    self
  }

  pub fn short(mut self, short: &str) -> Self {
    self.short = Some(short.to_string());
    self
  }

  pub fn default(mut self, default: &str) -> Self {
    self.default = Some(default.to_string());
    self
  }

  pub fn default_help(mut self, default_help: &str) -> Self {
    self.default_help = Some(default_help.to_string());
    self
  }

  pub fn optional(mut self, optional: bool) -> Self {
    self.optional = optional;
    self
  }

  pub fn help(mut self, help: &str) -> Self {
    self.help = help.to_string();
    self
  }

  pub fn format_help(descs: &[&ArgDesc], include_captions: bool) -> String {
    let (len_name, len_short, len_default) = descs.iter().fold((0, 0, 0), |(len_name, len_short, len_default), d| {
      if let Some(1) = d.index.as_ref() {
        (len_name, len_short, len_default)
      } else {
        (
          usize::max(d.name.len(), len_name),
          usize::max(d.short.as_ref().map(|s| s.len()).unwrap_or(0), len_short),
          usize::max(
            d.default_help.as_ref().or(d.default.as_ref()).map(|s| s.len()).unwrap_or(0),
            len_default,
          ),
        )
      }
    });

    let len_name = usize::max("Name".len() + 2, len_name + 4);
    let len_short = usize::max("Short".len() + 2, len_short + 3);
    let len_optional = match descs.iter().fold(false, |o, d| o | d.optional) {
      true => "[opt]".len() + 2,
      false => 0,
    };
    let len_default = match len_default {
      0 => 0,
      l => l + 2,
    };
    let len_sum = len_name + len_short + len_optional + len_default;

    let format_single_line = |d: &ArgDesc| {
      format!(
        "{name:<0$}{short:<1$}{opt:<2$}{def:<3$}{help}\n",
        len_name,
        len_short,
        len_optional,
        len_default,
        name = match d.index.filter(|i| *i == 0).is_some() {
          true => d.name.to_string(),
          false => format!("--{}", d.name),
        },
        short = match d.short.as_ref() {
          Some(short) => format!("-{}", short),
          None => Default::default(),
        },
        opt = match d.optional {
          true => "[opt]",
          false => "",
        },
        def = d.default_help.as_ref().or(d.default.as_ref()).map(|s| s.as_str()).unwrap_or(""),
        help = d.help.lines().next().unwrap()
      )
    };

    let mut help = match include_captions {
      true => format!(
        "{name:<0$}{short:<1$}{opt:<2$}{def:<3$}\n",
        len_name,
        len_short,
        len_optional,
        len_default,
        name = "name",
        short = "short",
        opt = match len_optional {
          0 => "",
          _ => "optional",
        },
        def = match len_default {
          0 => "",
          _ => "default",
        }
      ),
      false => String::new(),
    };

    if include_captions {
      for _ in 0..len_name - 1 {
        help.push('-');
      }
      help.push(' ');
      for _ in 0..len_short - 1 {
        help.push('-');
      }
      help.push(' ');
      if len_optional > 0 {
        for _ in 0..len_optional - 1 {
          help.push('-');
        }
        help.push(' ');
      }
      if len_default > 0 {
        for _ in 0..len_default - 1 {
          help.push('-')
        }
        help.push(' ');
      }
    }
    help.push('\n');

    for d in descs.iter() {
      if d.help.lines().count() > 1 {
        help.push_str(&format_single_line(d));
        for l in d.help.lines().skip(1) {
          help.push_str(&format!("{pad:<0$}{help}\n", len_sum, pad = "", help = l));
        }
      } else {
        help.push_str(&format_single_line(d));
      }
    }

    help
  }
}

/// Base argument trait
///
/// #Requires
/// - [get_desc](trait.Arg.html#tymethod.parse)
///
/// #Provides
/// - [parse](trait.Arg.html#method.parse)
/// - [complete](trait.Arg.html#method.complete)
///
/// An argument that implements this trait can be used by [Command](../trait.Command.html) to parse an argument string.
///
/// User defined implementation of [complete](trait.Arg.html#method.complete) may provide additional argument value completion.
/// No completions are given by the default implementation of [complete](trait.Arg.html#method.complete).
pub trait Arg {
  /// Gets the argument description
  ///
  /// Contains meta information used for parsing and user manual
  fn get_desc(&self) -> &ArgDesc;

  fn parse<'a>(
    &'a self,
    argsv: &'a [&'a str],
    argi: usize,
    completions: &mut Option<Vec<matches::Completion>>,
  ) -> Option<(usize, matches::Parsed)> {
    let argsv = &argsv[argi..];

    let desc = self.get_desc();
    let argname = desc.name.clone();
    let hint = ArgDesc::format_help(&[&desc], false);

    // empty input is completed with the argument name
    if argsv.is_empty() || argsv[0].trim().is_empty() {
      if let Some(completions) = completions.as_mut() {
        completions.push(Completion {
          index: argi,
          completed: format!("--{}", argname),
          hint,
        });
      }
      return None;
    }

    let mut is_indexed = false;
    let name = argsv[0].trim().to_string();
    // parses argument name
    // if it can not be parsed completely, but is a prefix we add a completion entry and return
    if name.starts_with("--") {
      if name[2..] != argname {
        if argname.starts_with(&name[2..]) {
          if let Some(completions) = completions.as_mut() {
            completions.push(Completion {
              index: argi,
              completed: format!("--{}", argname),
              hint,
            });
          }
        }
        return None;
      }
    }
    // parse argument name as short
    // if it can not be parsed completely, but is a prefix we add a completion and return
    else if name.starts_with("-") {
      if name == "-" {
        if let Some(completions) = completions.as_mut() {
          completions.push(Completion {
            index: argi,
            completed: format!("--{}", argname.to_string()),
            hint: hint.clone(),
          });
        }
      }
      if let Some(short) = desc.short.as_ref() {
        if name[1..] != *short {
          if let Some(completions) = completions.as_mut() {
            if short.starts_with(&name[1..]) {
              completions.push(Completion {
                index: argi,
                completed: format!("-{}", short),
                hint,
              });
            }
          }
          return None;
        }
      }
    }
    // argument name is neithehr a complete name or a short
    // we only return if it is a non-indexed argument
    else {
      is_indexed = true;
      if let None = desc.index.as_ref() {
        return None;
      }
    }

    let i_value = if is_indexed { 0 } else { 1 };
    let value = match i_value < argsv.len() {
      true => argsv[i_value].trim().to_string(),
      false => String::new(),
    };

    if let Some(completions) = completions.as_mut() {
      for completed in self.complete_variants_from_prefix(&value).into_iter() {
        let hint = hint.clone();
        completions.push(Completion {
          index: argi + i_value,
          completed,
          hint,
        });
      }
    }

    match self.validate_parsed_value(&value) {
      true => Some((argi + i_value, matches::Parsed { name, value })),
      false => None,
    }
  }

  // TODO: make use of that in parse
  fn validate_parsed_value(&self, _value: &str) -> bool {
    true
  }

  fn complete_variants_from_prefix(&self, _prefix: &str) -> Vec<String> {
    vec![]
  }

  fn get_match<'a>(&self, matches: &'a Matches) -> Option<&'a str> {
    matches.value_of(&self.get_desc().name)
  }
}

pub trait Convert<T>: Arg {
  fn from_match(&self, matches: &Matches) -> Option<T>;
}

mod matches;

pub use matches::Completion;
pub use matches::Matches;

mod bool;
mod commandname;
mod file;
mod ident;
mod num;

pub use self::bool::Bool;
pub use commandname::CommandName;
pub use file::File;
pub use ident::Ident;
pub use num::*;
