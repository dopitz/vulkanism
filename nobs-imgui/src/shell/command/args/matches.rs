use crate::shell::command::args::Arg;
use crate::shell::command::args::ArgDesc;

use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Parsed {
  pub name: String,
  pub value: String,
}

#[derive(Clone, Debug)]
pub struct Completion {
  pub index: usize,
  pub completed: String,
  pub hint: String,
}

impl Completion {
  pub fn complete(&self, input: &str) -> String {
    let mut args = Matches::split_args(input);

    match self.index < args.len() {
      true => {
        let mut arg = args[self.index];
        let arg = match arg.chars().position(|c| c != ' ') {
          Some(leading_whitespaces) => format!("{}{}", &arg[..leading_whitespaces], self.completed),
          None => format!(" {}", self.completed.clone()),
        };

        args[self.index] = &arg;
        args.iter().fold(String::new(), |s, a| format!("{}{}", s, a))
      }
      false => {
        let s = args.iter().fold(String::new(), |s, a| format!("{}{}", s, a));
        format!("{} {}", s, self.completed)
      }
    }
  }
}

#[derive(Debug)]
pub struct Matches {
  //input: &'a str,
  parsed: Vec<Parsed>,
}

impl Matches {
  fn split_args(input: &str) -> Vec<&str> {
    let mut args = Vec::new();

    let scan_space = |o| match input.chars().skip(o).position(|c| c != ' ') {
      Some(p) => o + p,
      None => input.len(),
    };

    let scan_char = |o, c| match input.chars().skip(o).position(|x| x == c) {
      Some(p) => o + p,
      None => input.len(),
    };

    let mut b = 0;
    loop {
      let e = scan_space(b);
      let e = if e < input.len() {
        match input.chars().nth(e) {
          Some('"') => usize::min(input.len(), scan_char(e + 1, '"') + 1),
          _ => scan_char(e, ' '),
        }
      } else {
        scan_char(e, ' ')
      };

      args.push(&input[b..e]);

      b = e;
      if b == input.len() {
        break;
      }
    }

    args
  }

  fn parse_args(argsv: &[&str], args: &[&dyn Arg], completions: &mut Option<Vec<Completion>>) -> Option<Vec<Parsed>> {
    // TODO: make sure there is always exatly ONE arg::CommandName...
    let i_cmdname = args.iter().position(|a| a.get_desc().index.filter(|i| *i == 0).is_some()).unwrap();
    let cmdname = args[i_cmdname].get_desc().name.as_str();

    // parse commandname
    if argsv[0] != cmdname {
      if argsv.is_empty() || (argsv.len() == 1 && cmdname.starts_with(argsv[0])) {
        if let Some(completions) = completions.as_mut() {
          completions.push(Completion {
            index: 0,
            completed: cmdname.to_string(),
            hint: ArgDesc::format_help(&[args[i_cmdname].get_desc()], false),
          });
        }
      }
      return None;
    }

    let mut parsed: Vec<Option<Parsed>> = vec![None; args.len()];
    parsed[i_cmdname] = Some(Parsed {
      name: cmdname.to_string(),
      value: cmdname.to_string(),
    });

    let mut argi = 1;

    loop {
      //let mut cnone = None;
      //let completions = match argi < argsv.len() - 2 {
      //  true => &mut cnone,
      //  false => completions,
      //};

      if let Some((i, (next_argi, p))) = args
        .iter()
        .enumerate()
        .filter(|(i, _)| parsed[*i].is_none())
        //.find_map(|(i, a)| a.parse(&argsv, argi, completions).map(|p| (i, p)))
        .find_map(|(i, a)| a.parse(&argsv, argi, completions).map(|p| (i, p)))
      {
        parsed[i] = Some(p);
        argi = next_argi;

        println!("{:?}", parsed);
      } else {
        break;
      }
    }

    // assign default values to arguments, that are not flagged as optional
    args
      .iter()
      .zip(parsed.iter_mut())
      .enumerate()
      .filter(|(_, (a, p))| !a.get_desc().optional && a.get_desc().default.is_some() && p.is_none())
      .for_each(|(i, (a, p))| {
        *p = Some(Parsed {
          name: String::new(),
          value: a.get_desc().default.as_ref().unwrap().clone(),
        });
      });

    // make sure all arguments are eithehr optional or have a parse result
    if args.iter().zip(parsed.iter()).all(|(a, p)| a.get_desc().optional || p.is_some()) {
      // return array of parsed arguments with same ordering as specified in the input
      Some(parsed.into_iter().filter_map(|p| p).collect())
    } else {
      None
    }
  }

  pub fn new(input: &str, args: Vec<&dyn Arg>, completions: &mut Option<Vec<Completion>>) -> Option<Self> {
    let argsv = Self::split_args(input);

    let mut args = args;
    args.sort_by(|a, b| a.get_desc().index.cmp(&b.get_desc().index));

    Self::parse_args(&argsv, &args, completions).map(|parsed| Self { parsed })
  }

  pub fn value_of(&self, name: &str) -> Option<&str> {
    self.parsed.iter().find(|p| name == p.name).map(|p| p.value.as_str())
  }
}
