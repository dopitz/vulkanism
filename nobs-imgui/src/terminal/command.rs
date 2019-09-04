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

  prefix_len: usize,
  complete_index: Option<usize>,
}

impl<S: Style> Shell<S> {
  pub fn new(term: Terminal<S>) -> Self {
    Shell {
      term,
      cmds: Default::default(),

      prefix_len: 0,
      complete_index: None,
    }
  }

  pub fn add_command(&mut self, cmd: Box<dyn Command>) {
    self.cmds.insert(cmd.get_name().to_owned(), cmd);
  }

  pub fn update<L: Layout>(&mut self, screen: &mut Screen<S>, layout: &mut L, focus: &mut SelectId) {
    match self.term.draw(screen, layout, focus) {
      Some(Event::TabComplete(shift)) => {
        let input = self.term.get_input();
        let prefix = input[..self.prefix_len].to_string();
        if let Some(completions) = self.cmds.iter().find_map(|(_, cmd)| cmd.complete(&prefix)) {
          println!("{:?}", self.complete_index);
          self.complete_index = match self.complete_index {
            None => match shift {
              false => Some(0),
              true => Some(completions.len() - 1),
            },
            Some(ci) => {
              let ci = ci as i32
                + match shift {
                  false => 1,
                  true => -1,
                };
              if ci < 0 || ci >= completions.len() as i32 {
                None
              } else {
                Some(ci as usize)
              }
            }
          };
          println!("{:?}", self.complete_index);

          println!("{:?}", &completions);
          if let Some(&ci) = self.complete_index.as_ref() {
            println!("complete: {} {}", ci, &completions[ci].complete());
            self.term.input_text(&completions[ci].complete());
          } else {
            self.term.input_text(&prefix);
          }
        }
      }
      Some(Event::InputChanged) => {
        let input = self.term.get_input();
        println!("{:?}", input);
        self.prefix_len = input.len();
        self.complete_index = None;

        if let Some(completions) = self.cmds.iter().find_map(|(_, cmd)| cmd.complete(&input)) {
          let mut s = completions.iter().fold(String::new(), |acc, c| format!("{}{}\n", acc, c.variant));
          s = format!("{}{}", s, "-------------");
          self.term.quickfix_text(&s);
        } else {
          self.term.quickfix_text("");
        }
      }
      Some(Event::InputSubmit(input)) => {
        println!("input: {:?}", input);
        for (_, cmd) in self.cmds.iter() {
          println!("{:?}", cmd.parse(&input));
        }
        self.prefix_len = 0;
        self.complete_index = None;
        self.term.quickfix_text("");
      }
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
      let args = self.match_args(s);

      let parsed = self
        .get_args()
        .iter()
        .zip(args.iter())
        .filter_map(|(a, sa)| {
          if a.parse(&s[sa.0..sa.1]).is_some() {
            Some(s[sa.0..sa.1].to_string())
          } else {
            None
          }
        })
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
  fn complete<'a>(&self, s: &'a str) -> Option<Vec<Completion<'a>>> {
    if self.get_name().starts_with(s) {
      Some(vec![Completion::new(&s[0..0], self.get_name().into())])
    } else if s.starts_with(self.get_name()) && !self.get_args().is_empty() {
      let args = self.match_args(s);
      println!("ccc: {:?} {}", args, s.len());
      if args.is_empty() {
        self.get_args()[0]
          .complete("")
          .map(|cs| cs.into_iter().map(|c| Completion::new(&s, c.variant)).collect())
      } else if args.len() < self.get_args().len() {
        let mut i = args.len() - 1;
        if args[i].1 < s.len() {
          self.get_args()[i + 1]
            .complete("")
            .map(|cs| cs.into_iter().map(|c| Completion::new(&s, c.variant)).collect())
        } else {
          self.get_args()[i]
            .complete(&s[args[i].0..args[i].1])
            .map(|cs| cs.into_iter().map(|c| Completion::new(&s[0..args[i].0], c.variant)).collect())
        }
      } else {
        None
      }
    } else {
      None
    }
  }
}

impl Command {
  fn match_args(&self, s: &str) -> Vec<(usize, usize)> {
    let mut matches = Vec::new();

    if s.len() > self.get_name().len() {
      let mut begin = match s[self.get_name().len()..].starts_with(" ") {
        true => None,
        false => Some(self.get_name().len()),
      };
      let mut quote = false;
      let mut escape = false;
      for (i, c) in s.chars().enumerate().skip(self.get_name().len()) {
        if !escape {
          if !quote && begin.is_none() && c == '\"' {
            quote = true;
            begin = Some(i);
          } else if quote && begin.is_some() && c == '\"' {
            quote = false;
            matches.push((begin.take().unwrap() + 1, i));
            escape = true;
          }
        }

        if !escape && !quote {
          if begin.is_none() && c != ' ' {
            begin = Some(i);
          } else if begin.is_some() && c == ' ' {
            matches.push((begin.take().unwrap(), i));
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
        matches.push((b, s.len()));
      }
    }

    matches
  }
}

#[derive(Debug)]
pub struct Completion<'a> {
  pub prefix: &'a str,
  pub variant: String,
}

impl<'a> Completion<'a> {
  pub fn new(prefix: &'a str, variant: String) -> Self {
    Self { prefix, variant }
  }

  pub fn complete(&self) -> String {
    format!("{}{}", self.prefix, self.variant)
  }
}

pub trait Parsable {
  fn parse(&self, s: &str) -> Option<Vec<String>>;
  fn complete<'a>(&self, s: &'a str) -> Option<Vec<Completion<'a>>>;
}

const ABCIDENTS: &[&str] = &["aaa", "abc", "bbb", "bcd", "ccc"];

pub struct ABCEnumArg {}

impl Parsable for ABCEnumArg {
  fn parse(&self, s: &str) -> Option<Vec<String>> {
    ABCIDENTS.iter().find(|i| &s == *i).map(|_| vec![s.into()])
  }
  fn complete<'a>(&self, s: &'a str) -> Option<Vec<Completion<'a>>> {
    let res = ABCIDENTS
      .iter()
      .filter(|i| i.starts_with(s))
      .map(|i| Completion::new(&s[0..0], i.to_owned().to_owned()))
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
