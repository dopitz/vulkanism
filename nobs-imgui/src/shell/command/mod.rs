pub mod args;
pub mod help;
pub mod source;
pub mod spawn;

use crate::shell::context::ContextBase;
use args::Arg;
use args::Completion;
use args::Matches;
use args::Parsed;

pub trait Command<C: ContextBase>: Send + Sync {
  fn get_args<'a>(&'a self) -> Vec<&'a dyn Arg>;

  fn run(&self, matches: &Matches, context: &mut C) -> Result<(), String>;

  fn get_commandname<'a>(&'a self) -> String {
    self
      .get_args()
      .iter()
      .find(|a| a.get_desc().index.filter(|i| *i == 0).is_some())
      .map(|a| a.get_desc().name.clone())
      .unwrap()
  }

  fn get_help(&self) -> String {
    let args = self.get_args();

    // headding (name of the command)
    let cmdname_arg = self
      .get_args()
      .iter()
      .find(|a| a.get_desc().index.filter(|i| *i == 0).is_some())
      .unwrap()
      .get_desc();
    let mut s = cmdname_arg.name.clone();
    s.push_str("\n  ");
    s.push_str(&cmdname_arg.help.replace("\n", "\n  "));
    s.push('\n');

    // example call (cmd <arg1> <arg2> [flags])
    let mut index_args = args
      .iter()
      .filter(|a| a.get_desc().index.filter(|i| *i != 0).is_some())
      .map(|a| (a.get_desc().index.unwrap(), a))
      .collect::<Vec<_>>();
    index_args.sort_by(|(a, _), (b, _)| a.cmp(b));
    let mut index_args = index_args.iter().map(|(_, a)| a.get_desc()).collect::<Vec<_>>();
    s.push('\n');
    s.push_str("Usage:\n");
    s.push_str("\n  ");
    s.push_str(&cmdname_arg.name);
    s.push(' ');
    for d in index_args.iter() {
      s.push('<');
      s.push_str(&d.name);
      s.push('>');
      s.push(' ');
    }
    s.push_str(" [options...]\n");

    let mut option_args = args
      .iter()
      .filter(|a| a.get_desc().index.is_none())
      .map(|a| a.get_desc())
      .collect::<Vec<_>>();
    option_args.sort_by(|a, b| a.name.cmp(&b.name));

    let mut sorted_args = Vec::with_capacity(1 + index_args.len() + option_args.len());
    sorted_args.append(&mut index_args);
    sorted_args.append(&mut option_args);

    s.push('\n');
    s.push_str("Arguments:\n");
    s.push_str("\n  ");
    s.push_str(&args::ArgDesc::format_help(sorted_args.as_slice(), true).replace("\n", "\n  "));
    s
  }

  fn parse<'a>(&'a self, s: &'a str, mut completions: Option<&mut Vec<Completion>>) -> Option<Vec<Parsed<'a>>> {
    let args = self.get_args();
    let mut parsed: Vec<Option<Parsed>> = vec![None; args.len()];
    let mut argorder = Vec::with_capacity(args.len());

    // TODO: make sure there is always exatly ONE arg::CommandName...
    // parse the command name and get argument string
    let mut pp = args
      .iter()
      .enumerate()
      .find(|(_, a)| a.get_desc().index.filter(|i| *i == 0).is_some())
      .and_then(|(i, a)| a.parse(s, 0, completions.as_mut()).map(|p| (i, p)));

    while let Some((i, p)) = pp {
      argorder.push(i);
      parsed[i] = Some(p.clone());

      // TODO: name clashes of arguments are handled during command/argument construction
      // it is possible that more than one argument parse successfully (unnamed arguments)
      // such arguments are ordered by the index of the argument descriptor
      // choosing the min element of the remaining unparsed arguments yields a unique result
      pp = args
        .iter()
        .enumerate()
        .filter(|(i, _)| parsed[*i].is_none())
        .filter_map(|(i, a)| {
          a.parse(s, p.replace_input.end, completions.as_mut())
            .map(|p| (a.get_desc().index, (i, p)))
        })
        .min_by(|(a, _), (b, _)| a.cmp(b))
        .map(|(_, x)| x);
    }

    // assign default values to arguments, that are not flagged as optional
    args
      .iter()
      .zip(parsed.iter_mut())
      .enumerate()
      .filter(|(_, (a, p))| !a.get_desc().optional && a.get_desc().default.is_some() && p.is_none())
      .for_each(|(i, (a, p))| {
        argorder.push(i);
        *p = Some(Parsed {
          input: "",
          replace_input: 0..0,
          name: "",
          value: a.get_desc().default.as_ref().unwrap().as_str(),
        });
      });

    // make sure all arguments are eithehr optional or have a parse result
    if args.iter().zip(parsed.iter()).all(|(a, p)| a.get_desc().optional || p.is_some()) {
      // return array of parsed arguments with same ordering as specified in the input
      Some(argorder.into_iter().map(|i| parsed[i].take().unwrap()).collect())
    } else {
      None
    }
  }
}

pub fn validate_command_def<C: ContextBase>(c: &Box<dyn Command<C>>) -> Result<(), &'static str> {
  let args = c.get_args();

  // the 0th argument is the name of the command
  args
    .iter()
    .find(|a| a.get_desc().index.filter(|i| *i == 0).is_some())
    .ok_or("Invalid Command: No Parameter with index == 0 defined.")?;

  // indices of arguments need to be in sequence and no duplicates
  let max_i = args.iter().filter_map(|a| a.get_desc().index).max().unwrap();
  if !(0..max_i)
    .into_iter()
    .all(|i| args.iter().filter_map(|a| a.get_desc().index).find(|j| i == *j).is_some())
  {
    Err("Invalid Command: Inconsistent argument indices. Indices must be consecutive starting from index 1.")?;
  }

  // make sure there are no name clashes with argument names and short names
  if !args.iter().all(|a| {
    let a = a.get_desc();
    let unique_name = args.iter().filter(|b| b.get_desc().name == a.name).count() <= 1;
    let unique_short = args
      .iter()
      .filter(|b| b.get_desc().name != a.name)
      .filter_map(|b| b.get_desc().short.as_ref())
      .all(|short| a.name != **short || a.short.is_none() || a.short.as_ref().filter(|s| *s != short).is_some());

    unique_name && unique_short
  }) {
    Err("Invalid Command: Argument names and short names need to be unique.")?;
  }

  Ok(())
}
