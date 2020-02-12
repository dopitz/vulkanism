use super::*;
use crate::style::Style;
use std::sync::Arc;
use std::sync::Mutex;

pub struct Shell<S: Style, C> {
  cmds: Arc<Mutex<Vec<Arc<dyn Command<S, C>>>>>,
}

impl<S: Style, C> Clone for Shell<S, C> {
  fn clone(&self) -> Self {
    Self { cmds: self.cmds.clone() }
  }
}

unsafe impl<S: Style, C> Send for Shell<S, C> {}

impl<S: Style, C> Shell<S, C> {
  pub fn new() -> Self {
    let sh = Self {
      cmds: Arc::new(Mutex::new(Vec::new())),
    };
    sh.add_command(Box::new(command::source::Cmd::new()));
    sh
  }

  fn add_command_inner(&self, cmd: Box<dyn Command<S, C>>) {
    let mut cmds = self.cmds.lock().unwrap();
    let name = cmd.get_name();
    if let Some(c) = cmds
      .iter()
      .find(|c| c.get_name().starts_with(name) || name.starts_with(c.get_name()))
    {
      println!("Command can not be added. Name conflict:\n{}\n{}", name, c.get_name());
    } else {
      cmds.push(cmd.into());
    }
    cmds.sort_by(|a, b| a.get_name().cmp(b.get_name()));
  }
  fn delete_command_inner(&self, name: &str) {
    let mut cmds = self.cmds.lock().unwrap();
    if let Some(p) = cmds.iter().position(|c| c.get_name() == name) {
      cmds.remove(p);
    }
  }
  fn update_help(&self) {
    self.delete_command_inner(command::help::Cmd::get_name());
    self.add_command_inner(Box::new(command::help::Cmd::new::<S, C>(&self.get_commands())));
  }

  pub fn add_command(&self, cmd: Box<dyn Command<S, C>>) {
    self.add_command_inner(cmd);
    self.update_help();
  }
  pub fn delete_command(&self, name: &str) {
    self.delete_command_inner(name);
    self.update_help();
  }

  pub fn exec(&self, c: &str, term: Terminal<S, C>, context: &mut C) {
    let exe = {
      self
        .cmds
        .lock()
        .unwrap()
        .iter()
        .find_map(|cmd| cmd.parse(c).map(|args| (cmd.clone(), args)))
    };
    if let Some((cmd, args)) = exe {
      cmd.run(args, term, context);
    }
  }

  pub fn get_commands(&self) -> Vec<Arc<dyn Command<S, C>>> {
    self.cmds.lock().unwrap().clone()
  }
}
