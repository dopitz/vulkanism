pub mod help;
pub mod source;

use super::arg;
use super::Shell;
use crate::style::Style;

pub trait Command<S: Style, C> {
  fn get_name(&self) -> &'static str;
  fn get_args<'a>(&'a self) -> Vec<&'a arg::Parsable>;

  fn get_info(&self) -> (&'static str, &'static str) {
    ("--", "--")
  }

  fn run(&self, args: Vec<String>, shell: Shell<S, C>, context: &mut C);
}

impl<S: Style, C> Command<S, C> {
  pub fn parse(&self, s: &str) -> Option<Vec<String>> {
    let cmd_args = self.get_args();

    if s.starts_with(self.get_name()) {
      let args = self.split_args(s);

      let parsed = cmd_args
        .iter()
        .zip(args.iter())
        .filter_map(|(a, sa)| if a.can_parse(&sa.1) { Some(sa.1.clone()) } else { None })
        .fold(vec![self.get_name().to_string()], |mut acc, arg| {
          acc.push(arg);
          acc
        });

      if parsed.len() == cmd_args.len() + 1 {
        Some(parsed)
      } else {
        None
      }
    } else {
      None
    }
  }
  // TODO cursor hint for completion when cursor not and end of input line
  pub fn complete(&self, s: &str) -> Option<Vec<arg::Completion>> {
    let cmd_args = self.get_args();

    // if the input is shorter than the command's name try to complete that
    if self.get_name().starts_with(s) {
      Some(vec![arg::Completion::new(0, self.get_name().into())])
    }
    // complete the arguments
    else if s.starts_with(self.get_name()) && !cmd_args.is_empty() {
      let args = self.split_args(s);
      // special case: input ends on whitespace
      if args.is_empty() && !cmd_args.is_empty() {
        cmd_args[0].complete("").map(|cs| {
          cs.into_iter()
            .map(|c| c.map_completed(|s| s.replace(" ", "\\ ")).prefix(s.to_string()))
            .collect()
        })
      }
      //
      else if args.len() <= cmd_args.len() {
        let i = args.len() - 1;
        if args[i].0.len() + args[i].1.len() < s.len() && i + 1 < cmd_args.len() {
          cmd_args[i + 1].complete("").map(|cs| {
            cs.into_iter()
              .map(|c| c.map_completed(|s| s.replace(" ", "\\ ")).prefix(s.to_string()))
              .collect()
          })
        } else {
          cmd_args[i].complete(&args[i].1).map(|cs| {
            cs.into_iter()
              .map(|c| c.map_completed(|s| s.replace(" ", "\\ ")).prefix(args[i].0.to_string()))
              .collect()
          })
        }
      } else {
        None
      }
    } else {
      None
    }
  }

  fn split_args<'a>(&self, s: &'a str) -> Vec<(&'a str, String)> {
    let mut matches = Vec::new();

    if s.len() > self.get_name().len() {
      // skip command name in input string
      let mut begin = match s[self.get_name().len()..].starts_with(" ") {
        true => None,
        false => Some(self.get_name().len()),
      };
      let mut quote = false;
      let mut escape = false;
      for (i, c) in s.chars().enumerate().skip(self.get_name().len()) {
        // argument between quotes
        if !escape {
          if !quote && begin.is_none() && c == '\"' {
            quote = true;
            begin = Some(i);
          } else if quote && begin.is_some() && c == '\"' {
            quote = false;
            let begin = begin.take().unwrap();
            matches.push((&s[..begin], s[begin + 1..i].to_string()));
            escape = true;
          }
        }

        if !escape && !quote {
          if begin.is_none() && c != ' ' {
            begin = Some(i);
          } else if begin.is_some() && c == ' ' {
            let begin = begin.take().unwrap();
            matches.push((&s[..begin], s[begin..i].to_string()));
          }
        }

        // escape whitespaces after backslash
        escape = false;
        if c == '\\' {
          escape = true;
        }
      }
      if let Some(mut b) = begin {
        if let Some('\"') = s.chars().nth(b) {
          b += 1;
        }
        matches.push((&s[..b], s[b..s.len()].to_string()));
      }
    }

    // remove escape backslashes
    matches.iter_mut().for_each(|s| s.1 = s.1.replace("\\ ", " ").replace("\\\\", "\\"));
    matches
  }
}