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

//fn parse_name<'a>(desc: &ArgDesc, s: &'a str, p: usize, completions: &mut Option<&mut &mut Vec<Completion>>) -> Option<&'a str> {
//  let end_on_space =
//    |offset| s[offset..].len() == desc.name.len() || s.chars().nth(offset + desc.name.len()).filter(|c| *c == ' ').is_some();
//
//  // argument name starts with "--"
//  // name clashes are resolved during construction
//  if s.starts_with("--") {
//    if s[2..].starts_with(&desc.name) && end_on_space(2) {
//      Some(&s[..2 + desc.name.len()])
//    } else {
//      if let Some(completions) = completions.as_mut() {
//        if desc.name.starts_with(&s[2..]) {
//          completions.push(Completion {
//            replace_input: p..p + s.len(),
//            completed: format!("--{}", desc.name),
//            hint: ArgDesc::format_help(&[desc], false),
//          });
//        }
//      }
//      None
//    }
//  }
//  //
//  // argument short starts with "-"
//  // name clashes are resolved during construction
//  else if s.starts_with("-") {
//    if desc.short.as_ref().filter(|short| s[1..].starts_with(short.as_str())).is_some() && end_on_space(1) {
//      Some(&s[..1 + desc.short.as_ref().unwrap().len()])
//    } else {
//      if let Some(completions) = completions.as_mut() {
//        if desc.short.as_ref().filter(|short| short.starts_with(&s[1..])).is_some() {
//          completions.push(Completion {
//            replace_input: p..p + s.len(),
//            completed: format!("-{}", desc.short.as_ref().unwrap()),
//            hint: ArgDesc::format_help(&[desc], false),
//          });
//        }
//        completions.push(Completion {
//          replace_input: p..p + s.len(),
//          completed: format!("--{}", desc.name),
//          hint: ArgDesc::format_help(&[desc], false),
//        });
//      }
//      None
//    }
//  }
//  //
//  // unnamed argument specifications must not start with "--" or "-"
//  else if !s.starts_with("-") && !s.starts_with("--") && !s.is_empty() && desc.index.filter(|i| *i > 0).is_some() {
//    Some("")
//  } else {
//    None
//  }
//}
//
//fn parse_value(s: &str) -> &str {
//  let s = s.trim();
//  if s.is_empty() {
//    s
//  }
//  // check single space directly, so that we don't need special treatment later
//  else if let Some(' ') = s.chars().next() {
//    ""
//  }
//  // "..." enclosed value
//  else if let Some('\"') = s.chars().next() {
//    match s.chars().skip(1).position(|c| c == '\"') {
//      Some(p) => &s[1..p + 1],
//      None => &s[1..],
//    }
//  }
//  // value with '\ ' spaces
//  else {
//    let mut p = 0;
//    while p < s.len() {
//      match s.chars().skip(p + 1).position(|c| c == ' ') {
//        Some(np) => p = np + p + 1, // + p + 1 because of the skip..
//        None => p = s.len(),
//      };
//      if !s.chars().nth(p - 1).filter(|c| *c == '\\').is_some() {
//        break;
//      }
//    }
//    &s[..p]
//  }
//}
//
//fn parse_next(offset: usize, s: &str) -> usize {
//  offset + s.chars().skip(offset).take_while(|c| *c == ' ').count()
//}

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
    argi: &mut usize,
    completions: &mut Option<Vec<matches::Completion>>,
  ) -> Option<matches::Parsed<'a>> {
    let argsv = &argsv[*argi..];

    let desc = self.get_desc();
    let argname = desc.name.clone();
    let hint = ArgDesc::format_help(&[&desc], false);

    if argsv.is_empty() {
      if let Some(completions) = completions.as_mut() {
        completions.push(Completion {
          index: *argi,
          completed: argname.to_string(),
          hint,
        });
      }
      return None;
    }

    let mut is_indexed = false;
    let name = argsv[0].trim();
    if name.starts_with("--") {
      if name[2..] != argname {
        if argname.starts_with(&name[2..]) {
          if let Some(completions) = completions.as_mut() {
            completions.push(Completion {
              index: *argi,
              completed: format!("--{}", argname),
              hint,
            });
          }
        }
        return None;
      }
    } else if name.starts_with("-") {
      if let Some(short) = desc.short.as_ref() {
        if name[1..] != *short {
          if short.starts_with(&name[1..]) {
            if let Some(completions) = completions.as_mut() {
              completions.push(Completion {
                index: *argi,
                completed: format!("-{}", short),
                hint,
              });
            }
          }
          return None;
        }
      }
    } else {
      is_indexed = true;
      if let None = self.get_desc().index.as_ref() {
        return None;
      }
    }

    let i_value = if is_indexed { 0 } else { 1 };
    let value = argsv[i_value].trim();

    if let Some(completions) = completions.as_mut() {
      for completed in self.complete_variants_from_prefix(value).into_iter() {
        let hint = hint.clone();
        completions.push(Completion {
          index: *argi + i_value,
          completed,
          hint,
        });
      }
    }

    *argi += i_value;

    match self.validate_parsed_value(value) {
      true => Some(matches::Parsed { name, value }),
      false => None,
    }
  }

  //  /// Tries to parse this argument from string
  //  ///
  //  /// # Arguments
  //  /// * `pdevice` - physical device handle
  //  /// * `device` - device handle
  //  ///
  //  /// # Returns
  //  /// - None, if the string could not be parsed.
  //  ///   This may be due to missmatching `name` or `short`
  //  /// - A [Parsed](struct.Parsed.html) containing the parsed value
  //  fn parse<'a>(&'a self, s: &'a str, offset: usize, mut completions: Option<&mut &mut Vec<Completion>>) -> Option<Parsed<'a>> {
  //    let input = s;
  //    let p = parse_next(offset, s);
  //
  //    // Special case, this is the command name
  //    let desc = self.get_desc();
  //    if let Some(0) = desc.index.as_ref() {
  //      if s[p..].starts_with(&desc.name) {
  //        let name = &s[p..desc.name.len()];
  //        let p = parse_next(p + name.len(), s);
  //
  //        Some(Parsed {
  //          input,
  //          replace_input: offset..p,
  //          name,
  //          value: name,
  //        })
  //      } else {
  //        // push a completion, if the prefix matches
  //        if let Some(completions) = completions.as_mut() {
  //          if desc.name.starts_with(&s[..s.len()]) {
  //            completions.push(Completion {
  //              replace_input: 0..s.len(),
  //              completed: desc.name.to_string(),
  //              hint: ArgDesc::format_help(&[desc], false),
  //            });
  //          }
  //        }
  //        None
  //      }
  //    }
  //    //
  //    // everything else are command arguments
  //    else if let Some(name) = parse_name(desc, &s[p..], p, &mut completions) {
  //      let vp = parse_next(p + name.len(), s);
  //      let value = parse_value(&s[vp..]);
  //      let np = parse_next(vp + value.len(), s);
  //
  //      // push completions of values, if name could be parsed correctly and is seperated with a space from the token before
  //      let index_arg = name.is_empty() && offset > 0 && s.chars().nth(offset - 1).filter(|c| *c == ' ').is_some();
  //      let space_after_arg_name = p + name.len() < vp;
  //      let no_space_after_value = vp + value.len() == np;
  //      if (index_arg || space_after_arg_name) && no_space_after_value {
  //        if let Some(completions) = completions.as_mut() {
  //          for c in self.complete_variants_from_prefix(value).into_iter() {
  //            completions.push(Completion {
  //              replace_input: vp..np,
  //              completed: c,
  //              hint: ArgDesc::format_help(&[desc], false),
  //            })
  //          }
  //        }
  //      }
  //
  //      Some(Parsed {
  //        input,
  //        replace_input: offset..p,
  //        name: &desc.name,
  //        value,
  //      })
  //    } else {
  //      None
  //    }
  //  }

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

//#[derive(Clone, Debug)]
//pub struct Completion {
//  pub replace_input: std::ops::Range<usize>,
//  pub completed: String,
//  pub hint: String,
//}
//
//impl Completion {
//  pub fn complete(&self, mut input: String) -> String {
//    if self.replace_input.start == self.replace_input.end {
//      input.push_str(&self.completed);
//    } else {
//      input.replace_range(self.replace_input.clone(), &self.completed);
//    }
//    input
//  }
//}

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
