use crate::shell::command::args::Arg;
use crate::shell::command::args::ArgDesc;

use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Parsed<'a> {
  pub name: &'a str,
  pub value: &'a str,
}

#[derive(Clone, Debug)]
pub struct Completion {
  pub index: usize,
  pub completed: String,
  pub hint: String,
}

impl Completion {
  pub fn complete(&self, input: &str) -> String {
    let args = Matches::split_args(input);
    let arg = args[self.index];

    match arg.chars().position(|c| c != ' ') {
      Some(leading_whitespaces) => format!("{}{}", &arg[..leading_whitespaces], self.completed),
      None => self.completed.clone(),
    }
  }
}

#[derive(Debug)]
pub struct Matches<'a> {
  input: &'a str,
  argsv: Vec<&'a str>,
  parsed: Option<Vec<Parsed<'a>>>,
}

impl<'a> Matches<'a> {
  fn split_args(input: &'a str) -> Vec<&'a str> {
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

  fn parse_args(argsv: &[&'a str], args: &[&'a dyn Arg], completions: &mut Option<Vec<Completion>>) -> Option<Vec<Parsed<'a>>> {
    //let mut argorder = Vec::with_capacity(args.len());

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
      name: cmdname,
      value: cmdname,
    });

    let mut argsv = &argsv[1..];
    let mut argi = 1;

    loop {
      args
        .iter()
        .enumerate()
        .filter(|(i, _)| parsed[*i].is_none())
        .map(|(i, a)| a.parse(argsv, &mut argi, completions));

      break;
    }

    //loop {
    //  let pp = args.iter().enumerate().filter(|(i, _)| parsed[*i].is_none()).find_map(|(i, a)| {
    //    let (p, av, c) = a.parse(argsv);
    //    match p {
    //      Some(p) => Some((i, p, av, c)),
    //      None => None,
    //    }
    //  });

    //  match pp {
    //    Some(i, p, av, c) => {
    //      parsed[i] = Some(p);
    //      argsv = av;

    //    }
    //  }

    //}

    //println!("{:?}", parsed);

    // TODO: make sure there is always exatly ONE arg::CommandName...
    // parse the command name and get argument string
    //let mut pp = args
    //  .iter()
    //  .enumerate()
    //  .find(|(_, a)| a.get_desc().index.filter(|i| *i == 0).is_some())
    //  .and_then(|(i, a)| a.parse(s, 0, completions.as_mut()).map(|p| (i, p)));

    //while let Some((i, p)) = pp {
    //  argorder.push(i);
    //  parsed[i] = Some(p.clone());

    //  println!("{:?}", parsed);

    //  // TODO: name clashes of arguments are handled during command/argument construction
    //  // it is possible that more than one argument parse successfully (unnamed arguments)
    //  // such arguments are ordered by the index of the argument descriptor
    //  // choosing the min element of the remaining unparsed arguments yields a unique result
    //  pp = args
    //    .iter()
    //    .enumerate()
    //    .filter(|(i, _)| parsed[*i].is_none())
    //    .filter_map(|(i, a)| {
    //      a.parse(s, p.replace_input.end, completions.as_mut())
    //        .map(|p| (a.get_desc().index, (i, p)))
    //    })
    //    .min_by(|(a, _), (b, _)| a.cmp(b))
    //    .map(|(_, x)| x);
    //}

    // assign default values to arguments, that are not flagged as optional
    args
      .iter()
      .zip(parsed.iter_mut())
      .enumerate()
      .filter(|(_, (a, p))| !a.get_desc().optional && a.get_desc().default.is_some() && p.is_none())
      .for_each(|(i, (a, p))| {
        *p = Some(Parsed {
          name: "",
          value: a.get_desc().default.as_ref().unwrap().as_str(),
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

  pub fn new(input: &'a str, args: Vec<&'a dyn Arg>, completions: &mut Option<Vec<Completion>>) -> Self {
    let argsv = Self::split_args(input);

    let mut args = args;
    args.sort_by(|a, b| a.get_desc().index.cmp(&b.get_desc().index));

    let parsed = Self::parse_args(&argsv, &args, completions);

    Self { input, argsv, parsed }
  }

  pub fn value_of(&self, name: &str) -> Option<&str> {
    self
      .parsed
      .as_ref()
      .and_then(|p| p.iter().find(|p| name == p.name).map(|p| p.value))
  }
}
