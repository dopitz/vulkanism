mod bool;
mod file;
mod ident;
mod num;

#[derive(Debug)]
pub struct Completion {
  score: i32,
  preview: String,
  completed: String,
  prefix: Option<String>,
  suffix: Option<String>,
}

impl Completion {
  pub fn new(score: i32, completed: String) -> Self {
    Self {
      score,
      preview: completed.clone(),
      completed,
      prefix: None,
      suffix: None,
    }
  }

  pub fn preview(mut self, pv: String) -> Self {
    self.preview = pv;
    self
  }
  pub fn prefix(mut self, pf: String) -> Self {
    self.prefix = self
      .prefix
      .map(|mut s| {
        s.insert_str(0, &pf);
        s
      })
      .or(Some(pf));
    self
  }
  pub fn suffix(mut self, sf: String) -> Self {
    self.suffix = self
      .suffix
      .map(|mut s| {
        s.push_str(&sf);
        s
      })
      .or(Some(sf));
    self
  }

  pub fn get_completed(&self) -> String {
    // TODO replace spaces with "\ "
    // only when required (nested call of get_complete in command...)
    format!(
      "{}{}{}",
      match self.prefix.as_ref() {
        Some(pf) => &pf,
        None => "",
      },
      self.completed,
      match self.suffix.as_ref() {
        Some(sf) => &sf,
        None => "",
      },
    )
  }

  pub fn map_completed<F: Fn(String) -> String>(mut self, f: F) -> Self {
    self.completed = f(self.completed);
    self
  }

  pub fn get_preview(&self) -> &str {
    &self.preview
  }
}

impl PartialEq for Completion {
  fn eq(&self, other: &Self) -> bool {
    self.score == other.score && self.completed == other.completed
  }
}

impl Eq for Completion {}

impl PartialOrd for Completion {
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    Some(self.cmp(other))
  }
}

impl Ord for Completion {
  fn cmp(&self, other: &Self) -> std::cmp::Ordering {
    match self.score.cmp(&other.score) {
      std::cmp::Ordering::Equal => self.completed.cmp(&other.completed),
      x => x,
    }
  }
}

pub trait Parsable {
  fn can_parse(&self, s: &str) -> bool;
  fn complete(&self, s: &str) -> Option<Vec<Completion>>;
}

pub trait Convert<T> {
  fn convert(&self, s: &str) -> Option<T>;

  fn convert_nth(&self, args: &[String], n: usize) -> Option<T> {
    if args.len() > n {
      self.convert(&args[n])
    } else {
      None
    }
  }
}

pub trait ConvertDefault<T>: Convert<T> {
  fn default(&self) -> Option<T>;

  fn convert_or_default(&self, s: &str) -> Option<T> {
    self.convert(s).or_else(|| self.default())
  }
  fn convert_nth_or_default(&self, args: &[String], n: usize) -> Option<T> {
    self.convert_nth(args, n).or_else(|| self.default())
  }
}

pub use self::bool::Bool;
pub use file::File;
pub use ident::Ident;
pub use num::*;

#[derive(Default)]
pub struct ArgumentDesc {
  pub index: usize,
  pub name: String,
  pub short: String,
  pub default: Option<String>,
  pub optional: bool,
  pub help: String,
}

impl ArgumentDesc {
  pub fn new() -> Self {
    Default::default()
  }

  pub fn index(mut self, i: usize) -> Self {
    self.index = i;
    self
  }

  pub fn name(mut self, name: &str) -> Self {
    self.name = name.to_string();
    self
  }

  pub fn short(mut self, short: &str) -> Self {
    self.short = short.to_string();
    self
  }

  pub fn default(mut self, default: &str) -> Self {
    self.default = Some(default.to_string());
    self
  }

  pub fn help(mut self, help: &str) -> Self {
    self.help = help.to_string();
    self
  }
}

pub trait Argument {
  fn get_desc(&self) -> &ArgumentDesc;

  /// Tries to parse this argument from
  fn parse<'a>(&self, s: &'a str) -> Option<(Parsed<'a>, &'a str)> {
    fn parse_name<'a>(desc: &ArgumentDesc, s: &'a str) -> Option<&'a str> {
      if s.starts_with("-") && s[1..].starts_with(&desc.short) {
        Some(&s[..1 + desc.short.len()])
      } else if s.starts_with("--") && s[2..].starts_with(&desc.name) {
        Some(&s[..2 + desc.name.len()])
      } else if !s.starts_with("-") && !s.starts_with("--") && desc.index > 0 {
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
          Some(p) => &s[1..p],
          None => &s[1..],
        }
      }
      // value with '\ ' spaces
      else {
        let mut p = 0;
        loop {
          match s
            .chars()
            .skip(p + 1)
            .position(|c| c == ' ')
            .filter(|p| s.chars().nth(p - 1).filter(|c| *c == '\\').is_some())
          {
            Some(np) => p = np,
            None => break,
          }
        }
        &s[..p]
      }
    }

    fn parse_next(offset: usize, s: &str) -> usize {
      s.chars().skip(offset).take_while(|c| *c != ' ').take_while(|c| *c != ' ').count()
    }

    let input = s;
    let p = parse_next(0, s);

    // Special case, this is the command name
    let desc = self.get_desc();
    if desc.index == 0 {
      if s[p..].starts_with(&desc.name) {
        let name = &s[p..desc.name.len()];
        let p = parse_next(p + name.len(), s);
        let next = &s[p..];
        Some((Parsed { input, name, value: "" }, next))
      } else {
        None
      }
    }
    //
    // everything else are command arguments
    else if let Some(name) = parse_name(self.get_desc(), &s[p..]) {
      let p = parse_next(p + name.len(), s);
      let value = parse_value(&s[p..]);
      let p = parse_next(p + value.len(), s);
      let next = &s[p..];

      Some((Parsed { input, name, value }, next))
    } else {
      None
    }
  }
}

pub struct Parsed<'a> {
  pub input: &'a str,
  pub name: &'a str,
  pub value: &'a str,
}

pub struct Completer {
  pub quickfix: String,
  pub completed: String,
}

pub struct CommandName {
  desc: ArgumentDesc,
}

impl CommandName {
  pub fn new(name: &str) -> Self {
    Self {
      desc: ArgumentDesc::new().index(0).name(name),
    }
  }
}

impl Argument for CommandName {
  fn get_desc<'a>(&'a self) -> &'a ArgumentDesc {
    &self.desc
  }
}
