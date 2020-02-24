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

  pub fn optional(mut self, optional: bool) -> Self {
    self.optional = optional;
    self
  }

  pub fn help(mut self, help: &str) -> Self {
    self.help = help.to_string();
    self
  }
}

fn parse_name<'a>(desc: &ArgDesc, s: &'a str) -> Option<&'a str> {
  // argument name starts with "--"
  // name clashes are resolved during construction
  if s.starts_with("--") && s[2..].starts_with(&desc.name) {
    Some(&s[..2 + desc.name.len()])
  }
  //
  // argument short starts with "-"
  // name clashes are resolved during construction
  else if s.starts_with("-") && desc.short.as_ref().filter(|short| s[1..].starts_with(short.as_str())).is_some() {
    Some(&s[..1 + desc.short.as_ref().unwrap().len()])
  }
  //
  // unnamed argument specifications must not start with "--" or "-"
  else if !s.starts_with("-") && !s.starts_with("--") && desc.index.filter(|i| *i > 0).is_some() {
    Some("")
  } else {
    None
  }
}

fn parse_value(s: &str) -> &str {
  let s = s.trim();
  if s.is_empty() {
    s
  }
  // check single space directly, so that we don't need special treatment later
  else if let Some(' ') = s.chars().next() {
    ""
  }
  // "..." enclosed value
  else if let Some('\"') = s.chars().next() {
    match s.chars().skip(1).position(|c| c == '\"') {
      Some(p) => &s[1..p + 1],
      None => &s[1..],
    }
  }
  // value with '\ ' spaces
  else {
    let mut p = 0;
    while p < s.len() {
      match s.chars().skip(p + 1).position(|c| c == ' ') {
        Some(np) => p = np + p + 1, // + p + 1 because of the skip..
        None => p = s.len(),
      };
      if !s.chars().nth(p - 1).filter(|c| *c == '\\').is_some() {
        break;
      }
    }
    &s[..p]
  }
}

fn parse_next(offset: usize, s: &str) -> usize {
  offset + s.chars().skip(offset).take_while(|c| *c == ' ').count()
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

  /// Tries to parse this argument from string
  ///
  /// # Arguments
  /// * `pdevice` - physical device handle
  /// * `device` - device handle
  ///
  /// # Returns
  /// - None, if the string could not be parsed.
  ///   This may be due to missmatching `name` or `short`
  /// - A [Parsed](struct.Parsed.html) containing the parsed value
  fn parse<'a>(&self, s: &'a str, offset: usize, mut completions: Option<&mut &mut Vec<Completion>>) -> Option<Parsed<'a>> {
    let input = s;
    let p = parse_next(offset, s);

    // Special case, this is the command name
    let desc = self.get_desc();
    if let Some(0) = desc.index.as_ref() {
      if s[p..].starts_with(&desc.name) {
        let name = &s[p..desc.name.len()];
        let p = parse_next(p + name.len(), s);
        let next = &s[p..];

        Some(Parsed {
          input,
          replace_input: offset..p,
          name,
          value: name,
        })
      } else {
        // push a completion, if the prefix matches
        if let Some(completions) = completions.as_mut() {
          if desc.name.starts_with(&s[..s.len()]) {
            completions.push(Completion {
              replace_input: 0..s.len(),
              completed: desc.name.to_string(),
            });
          }
        }
        None
      }
    }
    //
    // everything else are command arguments
    else if let Some(name) = parse_name(desc, &s[p..]) {
      let p = parse_next(p + name.len(), s);
      let value = parse_value(&s[p..]);
      let p = parse_next(p + value.len(), s);
      let next = &s[p..];
      Some(Parsed {
        input,
        replace_input: offset..p,
        name,
        value,
      })
    } else {
      None
    }
  }

  /// Get completions for the parsed argument value
  fn complete(&self, s: &str, offset: usize) -> Vec<Completion> {
    println!("{:?}, {}", s, offset);

    let input = s;
    let p = parse_next(offset, s);

    println!("{:?}, {}", &s[p..], p);

    let desc = self.get_desc();
    let name = parse_name(desc, &s[p..]);

    println!("{:?}", name);

    match name {
      Some(name) => {
        let p = parse_next(p + name.len(), s);
        let value = parse_value(&s[p..]);

        self
          .complete_variants_from_prefix(value)
          .into_iter()
          .map(|completed| Completion {
            replace_input: p..input.len(),
            completed,
          })
          .collect()
      }
      None => {
        println!("XXXXXXXXXXXXXXxx");
        if format!("--{}", desc.name).starts_with(&s[p..])
          || desc
            .short
            .as_ref()
            .filter(|short| format!("-{}", short).starts_with(&s[p..]))
            .is_some()
        {
          vec![Completion {
            replace_input: p..input.len(),
            completed: format!("--{}", desc.name),
          }]
        } else {
          vec![]
        }
      }
    }
  }

  fn complete_variants_from_prefix(&self, prefix: &str) -> Vec<String> {
    vec![]
  }
}

/// Parsed argument
#[derive(Clone, Debug)]
pub struct Parsed<'a> {
  /// The original input string, contianing the argument `name`, `value` and all intermediate and trailing whitespaces up to the next argument.
  pub input: &'a str,
  pub replace_input: std::ops::Range<usize>,
  /// The argument name as specified in the input string, with trimmed whitespaces.
  pub name: &'a str,
  /// The argument value as specified in the input string, with trimmed whitespaces.
  pub value: &'a str,
}

#[derive(Clone, Debug)]
pub struct Completion {
  pub replace_input: std::ops::Range<usize>,
  pub completed: String,
}

mod commandname;
mod file;
mod ident;

pub use commandname::CommandName;
pub use file::File;
pub use ident::Ident;
