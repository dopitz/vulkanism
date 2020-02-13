use crate::shell::command::help;
use crate::shell::command::source;
use crate::shell::Command;
use crate::shell::Context;
use std::sync::Arc;
use std::sync::Mutex;

struct ShellExec<C: Context> {
  pub cmd: Arc<dyn Command<C>>,
  pub args: Vec<String>,
}

pub struct Shell<C: Context> {
  cmds: Arc<Mutex<Vec<Arc<dyn Command<C>>>>>,
  exec: Arc<Mutex<Option<std::thread::JoinHandle<()>>>>,
}

impl<C: Context> Clone for Shell<C> {
  fn clone(&self) -> Self {
    Self {
      cmds: self.cmds.clone(),
      exec: self.exec.clone(),
    }
  }
}

unsafe impl<C: Context> Send for Shell<C> {}

impl<C: Context> Shell<C> {
  pub fn new() -> Self {
    let sh = Self {
      cmds: Arc::new(Mutex::new(Vec::new())),
      exec: Arc::new(Mutex::new(None)),
    };
    sh.add_command(Box::new(source::Cmd::new()));
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
    self.delete_command_inner(help::Cmd::get_name());
    self.add_command_inner(Box::new(help::Cmd::new(&self.get_commands())));
  }

  pub fn add_command(&self, cmd: Box<dyn Command<C>>) {
    self.add_command_inner(cmd);
    self.update_help();
  }
  pub fn delete_command(&self, name: &str) {
    self.delete_command_inner(name);
    self.update_help();
  }

  pub fn get_commands(&self) -> Vec<Arc<dyn Command<C>>> {
    self.cmds.lock().unwrap().clone()
  }

  pub fn exec(&self, s: &str, context: &mut C) {
    let exe = self
      .cmds
      .lock()
      .unwrap()
      .iter()
      .find_map(|cmd| cmd.parse(s).map(|args| (cmd.clone(), args)));

    if let Some((cmd, args)) = exe {
      cmd.run(args, context);
    }
  }
  pub fn has_exec(&self) -> bool {
    self.exec.lock().unwrap().is_some()
  }
}

impl<C: 'static + Clone + Send + Context> Shell<C> {
  pub fn exec_async(&self, s: &str, context: &mut C) {
    let exe = self
      .cmds
      .lock()
      .unwrap()
      .iter()
      .find_map(|cmd| cmd.parse(s).map(|args| (cmd.clone(), args)));

    if let Some((cmd, args)) = exe {
      let mut context = context.clone();
      *self.exec.lock().unwrap() = Some(std::thread::spawn(move || {
        cmd.run(args, &mut context);
        *context.get_shell().exec.lock().unwrap() = None;
      }))
    }
  }
}
