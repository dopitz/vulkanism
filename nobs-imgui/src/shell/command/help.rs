use crate::shell::command::args;
use crate::shell::command::args::Parsed;
use crate::shell::context::Context;
use crate::shell::Command;

#[derive(Clone)]
pub struct Cmd {
  thisname: args::CommandName,
  cmd: args::Ident,
}

impl<C: Context> Command<C> for Cmd {
  fn get_args<'a>(&'a self) -> Vec<&'a dyn args::Arg> {
    vec![&self.thisname, &self.cmd]
  }

  fn run(&self, args: &[Parsed], context: &mut C) {
    println!("AOEUAOEUAOEU");
    context.println("list of comands:");
    //if args.len() == 1 {
    //  println!("AOEUAOEUAOEU");
    //  let s = context.get_shell();
    //  println!("AOEUAOEUAOEU");
    //  let cmds = s.get_commands();
    //  println!("AOEUAOEUAOEU");
    //  let w = cmds.iter().fold(0, |w, c| usize::max(w, c.get_commandname().len()));
    //  println!("AOEUAOEUAOEU");

    //  context.println("list of comands:");
    //  for c in context.get_shell().get_commands().iter() {
    //    let mut n = c.get_commandname().to_string();
    //    while n.len() < w {
    //      n.push(' ');
    //    }
    //    //context.println(&format!("  {} -   {}", n, c.get_help().0));
    //  }
    //} else if let Some(cmd) = context.get_shell().get_commands().iter().find(|c| c.get_commandname() == args[1]) {
    //  let (short, desc) = cmd.get_help();
    //  context.println(&format!("{} - {}\n----------------------\n{}", cmd.get_commandname(), short, desc));
    //}
  }

  //fn run(&self, args: Vec<String>, context: &mut C) {
  //  if args.len() == 1 {
  //    println!("AOEUAOEUAOEU");
  //    let s = context.get_shell();
  //    println!("AOEUAOEUAOEU");
  //    let cmds = s.get_commands();
  //    println!("AOEUAOEUAOEU");
  //    let w = cmds
  //      .iter()
  //      .fold(0, |w, c| usize::max(w, c.get_name().len()));
  //    println!("AOEUAOEUAOEU");

  //    context.println("list of comands:");
  //    for c in context.get_shell().get_commands().iter() {
  //      let mut n = c.get_name().to_string();
  //      while n.len() < w {
  //        n.push(' ');
  //      }
  //      context.println(&format!("  {} -   {}", n, c.get_info().0));
  //    }
  //  } else if let Some(cmd) = context.get_shell().get_commands().iter().find(|c| c.get_name() == args[1]) {
  //    let (short, desc) = cmd.get_info();
  //    context.println(&format!("{} - {}\n----------------------\n{}", cmd.get_name(), short, desc));
  //  }
  //}
}

impl Cmd {
  pub fn new<C: Context>(cmds: &Vec<std::sync::Arc<dyn Command<C>>>) -> Self {
    let vars = cmds.iter().map(|c| c.get_commandname().to_string()).collect::<Vec<_>>();
    let cmds = vars.iter().map(|v| v.as_str()).collect::<Vec<_>>();

    Self {
      thisname: args::CommandName::new("help"),
      cmd: args::Ident::new(args::ArgDesc::new("command").index(0), &[cmds.as_slice()], None, true),
    }
  }

  pub fn update<C: Context>(&mut self, cmds: &Vec<std::sync::Arc<dyn Command<C>>>) {
    let vars = cmds.iter().map(|c| c.get_commandname().to_string()).collect::<Vec<_>>();
    let cmds = vars.iter().map(|v| v.as_str()).collect::<Vec<_>>();
    self.cmd = args::Ident::new(args::ArgDesc::new("command").index(0), &[cmds.as_slice()], None, true);
  }

  pub fn get_name() -> &'static str {
    "help"
  }
}
