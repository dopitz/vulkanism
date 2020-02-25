pub mod args;
pub mod help;
pub mod source;
pub mod spawn;

use crate::shell::Context;
use args::Arg;
use args::Completion;
use args::Parsed;

pub trait Command<C: Context>: Send + Sync {
  fn get_args<'a>(&'a self) -> Vec<&'a dyn Arg>;

  fn run(&self, args: &[Parsed], context: &mut C);

  fn get_commandname<'a>(&'a self) -> String {
    self
      .get_args()
      .iter()
      .find(|a| a.get_desc().index.filter(|i| *i == 0).is_some())
      .map(|a| a.get_desc().name.clone())
      .unwrap()
  }

  fn get_help(&self) -> String {
    let (len_short, len_name) = self.get_args().iter().fold((0, 0), |(s, n), a| {
      let d = a.get_desc();
      (
        usize::max(d.short.as_ref().map(|s| s.len()).unwrap_or(0), s),
        usize::max(d.name.len(), n),
      )
    });

    let len_short = len_short + 2;
    let len_name = len_name + 3;

    let h = format!(
      "{name:>0$} {short:>1$} {help}",
      len_name,
      len_short,
      name = "Name",
      short = "Short",
      help = "Help"
    );
    let h = format!(
      "{s}\n--{name:>0$} -{short:>1$} {help}",
      len_name,
      len_short,
      s = h,
      name = "Name",
      short = "Short",
      help = "Help"
    );

    h
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
      .find(|(i, a)| a.get_desc().index.filter(|i| *i == 0).is_some())
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
        .filter(|(i, a)| parsed[*i].is_none())
        .filter_map(|(i, a)| a.parse(s, p.replace_input.end, completions.as_mut()).map(|p| (a.get_desc().index, (i, p))))
        .min_by(|(a, _), (b, _)| a.cmp(b))
        .map(|(_, x)| x);
    }

    // assign default values to arguments, that are not flagged as optional
    args
      .iter()
      .zip(parsed.iter_mut())
      .enumerate()
      .filter(|(i, (a, p))| !a.get_desc().optional && a.get_desc().default.is_some() && p.is_none())
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
