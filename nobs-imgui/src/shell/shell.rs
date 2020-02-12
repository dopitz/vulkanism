use super::*;
use crate::style::Style;
use std::sync::Arc;
use std::sync::Mutex;

pub struct Shell<C: Context> {
  cmds: Arc<Mutex<Vec<Arc<dyn Command<C>>>>>,
}

impl<C: Context> Clone for Shell<C> {
  fn clone(&self) -> Self {
    Self { cmds: self.cmds.clone() }
  }
}

unsafe impl<C: Context> Send for Shell<C> {}

impl<C: Context> Shell<C> {
  pub fn new() -> Self {
    let sh = Self {
      cmds: Arc::new(Mutex::new(Vec::new())),
    };
    //TODO sh.add_command(Box::new(command::source::Cmd::new()));
    sh
  }

  fn add_command_inner(&self, cmd: Box<dyn Command<C>>) {
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
    self.add_command_inner(Box::new(command::help::Cmd::new(&self.get_commands())));
  }

  pub fn add_command(&self, cmd: Box<dyn Command<C>>) {
    self.add_command_inner(cmd);
    self.update_help();
  }
  pub fn delete_command(&self, name: &str) {
    self.delete_command_inner(name);
    self.update_help();
  }

  pub fn parse(&self, c: &str) -> Option<ShellExec<C>> {
    self
      .cmds
      .lock()
      .unwrap()
      .iter()
      .find_map(|cmd| cmd.parse(c).map(|args| ShellExec { cmd: cmd.clone(), args }))
  }

  pub fn get_commands(&self) -> Vec<Arc<dyn Command<C>>> {
    self.cmds.lock().unwrap().clone()
  }
}

pub struct ShellExec<C: Context> {
  cmd: Arc<dyn Command<C>>,
  args: Vec<String>,
}

impl<C: Context> ShellExec<C> {
  pub fn run(self, context: &mut C) {
    self.cmd.run(self.args, context);
  }
}
