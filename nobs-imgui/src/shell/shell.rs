use crate::shell::command;
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
    let name = &cmd.get_commandname();
    if let Some(c) = cmds
      .iter()
      .find(|c| c.get_commandname().starts_with(name) || name.starts_with(&c.get_commandname()))
    {
      println!("Command can not be added. Name conflict:\n{}\n{}", name, c.get_commandname());
    } else {
      cmds.push(cmd.into());
    }
    cmds.sort_by(|a, b| a.get_commandname().cmp(&b.get_commandname()));
  }
  fn delete_command_inner(&self, name: &str) {
    let mut cmds = self.cmds.lock().unwrap();
    if let Some(p) = cmds.iter().position(|c| c.get_commandname() == name) {
      cmds.remove(p);
    }
  }
  fn update_help(&self) {
    self.delete_command_inner(help::Cmd::get_name());
    self.add_command_inner(Box::new(help::Cmd::new(&self.get_commands())));
  }

  pub fn add_command(&self, cmd: Box<dyn Command<C>>) {
    match command::validate_command_def(&cmd) {
      Err(e) => println!("Command \"{}\" could not be added to the shell.\n{}", cmd.get_commandname(), e),
      Ok(_) => {
        self.add_command_inner(cmd);
        self.update_help();
      }
    }
  }
  pub fn delete_command(&self, name: &str) {
    self.delete_command_inner(name);
    self.update_help();
  }

  pub fn get_commands(&self) -> Vec<Arc<dyn Command<C>>> {
    self.cmds.lock().unwrap().clone()
  }

  pub fn exec(&self, s: &str, context: &mut C) {
    let cmd = self
      .cmds
      .lock()
      .unwrap()
      .iter()
      .find_map(|cmd| cmd.parse(s, None).map(|_| cmd.clone()));

    if let Some(cmd) = cmd {
      cmd.run(&cmd.parse(s, None).unwrap().into(), context);
    }
  }
}

impl<C: 'static + Clone + Send + Context> Shell<C> {
  pub fn has_exec(&self) -> bool {
    self.exec.lock().unwrap().is_some()
  }
  pub fn exec_async(&self, s: &str, context: &mut C) {
    let exe = self
      .cmds
      .lock()
      .unwrap()
      .iter()
      .find_map(|cmd| cmd.parse(s, None).map(|_| cmd.clone()));

    if let Some(cmd) = exe {
      let s = s.to_string();
      let mut context = context.clone();
      *self.exec.lock().unwrap() = Some(std::thread::spawn(move || {
        cmd.run(&cmd.parse(&s, None).unwrap().into(), &mut context);
        *context.get_shell().exec.lock().unwrap() = None;
      }))
    }
  }
}
