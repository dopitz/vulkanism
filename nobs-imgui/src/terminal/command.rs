use super::terminal::Event;
use super::Terminal;
use crate::select::SelectId;
use crate::style::Style;
use crate::window::*;

use regex::Regex;
use std::collections::BTreeMap;

pub struct Shell<S: Style> {
  term: Terminal<S>,

  cmds: BTreeMap<String, Box<dyn Command>>,
}

impl<S: Style> Shell<S> {
  pub fn new(term: Terminal<S>) -> Self {
    Shell {
      term,
      cmds: Default::default(),
    }
  }

  pub fn add_command(&mut self, cmd: Box<dyn Command>) {
    self.cmds.insert(cmd.get_name().to_owned(), cmd);
  }

  pub fn update<L: Layout>(&mut self, screen: &mut Screen<S>, layout: &mut L, focus: &mut SelectId) {
    match self.term.draw(screen, layout, focus) {
      Some(Event::InputChanged) => {
        for (_, cmd) in self.cmds.iter() {
          let complete = cmd.complete(&self.term.get_input());
          //println!("{:?}", complete);
          //if complete.len() > 1 {
          //  for cp in complete {
          //    self.term.println(&cp);
          //  }
          //  break;
          //}
        }
      }
      Some(Event::InputSubmit(input)) => {
        for (_, cmd) in self.cmds.iter() {
          println!("{:?}", cmd.parse(&input));
        }

      },
      _ => (),
    }
  }
}

pub trait Command {
  fn get_name(&self) -> &str;
  fn get_args(&self) -> &Vec<Box<dyn Parsable>>;
}

impl Parsable for Command {
  fn parse(&self, s: &str) -> Option<Vec<String>> {
    if s.starts_with(self.get_name()) {
      let args = Self::split_arguments(&s[self.get_name().len()..]);

      let parsed = self
        .get_args()
        .iter()
        .zip(args.iter())
        .filter_map(|(a, sa)| if a.parse(sa).is_some() { Some(sa.to_string()) } else { None })
        .fold(vec![self.get_name().to_string()], |mut acc, arg| {
          acc.push(arg);
          acc
        });

      if parsed.len() == self.get_args().len() + 1 {
        Some(parsed)
      } else {
        None
      }
    } else {
      None
    }
  }
  fn complete(&self, s: &str) -> Option<Vec<String>> {
    if self.get_name().starts_with(s) {
      Some(vec![self.get_name().into()])
    } else if s.starts_with(self.get_name()) {
      let args = Self::split_arguments(&s[self.get_name().len()..]);
      if !args.is_empty() && args.len() <= self.get_args().len() {
        let i = args.len() - 1;
        self.get_args()[i].complete(&args[i])
      } else {
        None
      }
    } else {
      None
    }
  }
}

impl Command {
  fn split_arguments(s: &str) -> Vec<String> {
    let mut matches = Vec::new();
    let mut begin = match s.starts_with(" ") {
      true => None,
      false => Some(0),
    };
    let mut quote = false;
    let mut escape = false;
    for (i, c) in s.chars().enumerate() {
      if !escape {
        if !quote && begin.is_none() && c == '\"' {
          quote = true;
          begin = Some(i);
        } else if quote && begin.is_some() && c == '\"' {
          quote = false;
          matches.push(s[begin.take().unwrap() + 1..i].into());
          escape = true;
        }
      }

      if !escape && !quote {
        if begin.is_none() && c != ' ' {
          begin = Some(i);
        } else if begin.is_some() && c == ' ' {
          matches.push(s[begin.take().unwrap()..i].into());
        }
      }

      escape = false;
      if c == '\\' {
        escape = true;
      }
    }
    if let Some(mut b) = begin {
      if let Some('\"') = s.chars().nth(b) {
        b += 1;
      }
      matches.push(s[b..].into());
    }

    matches
  }
}

pub trait Parsable {
  fn parse(&self, s: &str) -> Option<Vec<String>>;
  fn complete(&self, s: &str) -> Option<Vec<String>>;
}

const ABCIDENTS: &[&str] = &["aaa", "abc", "bbb", "bcd", "ccc"];

pub struct ABCEnumArg {}

impl Parsable for ABCEnumArg {
  fn parse(&self, s: &str) -> Option<Vec<String>> {
    ABCIDENTS.iter().find(|i| &s == *i).map(|_| vec![s.into()])
  }
  fn complete(&self, s: &str) -> Option<Vec<String>> {
    let res = ABCIDENTS
      .iter()
      .filter(|i| i.starts_with(s))
      .map(|i| i.to_owned().to_owned())
      .collect::<Vec<_>>();
    match res.is_empty() {
      true => None,
      false => Some(res),
    }
  }
}

pub struct ABCCommand {
  name: String,
  args: Vec<Box<dyn Parsable>>,
}

impl Command for ABCCommand {
  fn get_name(&self) -> &str {
    &self.name
  }
  fn get_args(&self) -> &Vec<Box<dyn Parsable>> {
    &self.args
  }
}

impl ABCCommand {
  pub fn new() -> Self {
    Self {
      name: "runabc".to_owned(),
      args: vec![Box::new(ABCEnumArg {}), Box::new(ABCEnumArg {})],
    }
  }
}
