use super::*;
use crate::components::textbox::Event;
use crate::select::SelectId;
use crate::style::Style;
use crate::window::*;
use crate::ImGui;
use std::sync::Arc;
use std::sync::Mutex;

#[derive(Clone, Copy)]
enum CompleteIndex {
  Input,
  Lcp,
  Index(usize),
}

#[derive(Clone, Copy)]
enum HistoryIndex {
  Input,
  Index(usize),
}

struct ShellImpl<S: Style, C> {
  term: TerminalWnd<S>,

  cmds: Vec<Arc<dyn Command<S, C>>>,

  history: Vec<String>,

  show_term: bool,
  prefix_len: usize,
  complete_index: CompleteIndex,
  history_index: HistoryIndex,
  shift: bool,
}

impl<S: Style, C> ShellImpl<S, C> {
  fn new(gui: &ImGui<S>) -> Self {
    let mut shell = Self {
      term: TerminalWnd::new(gui),

      cmds: Default::default(),

      history: Default::default(),

      show_term: false,
      prefix_len: 0,
      complete_index: CompleteIndex::Input,
      history_index: HistoryIndex::Input,
      shift: false,
    };
    shell
  }

  fn update<L: Layout>(
    &mut self,
    screen: &mut Screen<S>,
    layout: &mut L,
    focus: &mut SelectId,
  ) -> Option<(Arc<dyn Command<S, C>>, Vec<String>)> {
    let mut exe = None;
    if self.show_term {
      match self.term.draw(screen, layout, focus) {
        Some(Event::Changed) => {
          let input = self.term.get_input();
          self.prefix_len = input.len();
          self.complete_index = CompleteIndex::Input;

          if let Some(completions) = self.get_completions(&input) {
            let mut s = completions
              .iter()
              .fold(String::new(), |acc, c| format!("{}{}\n", acc, c.get_preview()));
            s = format!("{}{}", s, "-------------");
            self.term.quickfix_text(&s);
          } else {
            self.term.quickfix_text("");
          }
        }
        Some(Event::Enter(input)) => {
          exe = self.exec(&input);
          self.prefix_len = 0;
          self.complete_index = CompleteIndex::Input;
          self.term.quickfix_text("");
        }
        _ => (),
      }
    }

    let mut set_focus = None;
    for e in screen.get_events() {
      match e {
        vk::winit::Event::WindowEvent {
          event: vk::winit::WindowEvent::ReceivedCharacter(':'),
          ..
        } if !self.show_term => {
          self.show_term = true;
          self.term.focus(true);
          //set_focus = Some(true);
        }
        vk::winit::Event::WindowEvent {
          event:
            vk::winit::WindowEvent::KeyboardInput {
              input:
                vk::winit::KeyboardInput {
                  state: vk::winit::ElementState::Pressed,
                  virtual_keycode: Some(k),
                  ..
                },
              ..
            },
          ..
        } if self.show_term => match *k {
          vk::winit::VirtualKeyCode::Escape => {
            if self.term.get_input().is_empty() {
              self.show_term = false;
              set_focus = Some(false);
            } else {
              self.term.input_text("");
              self.term.quickfix_text("");
            }
          }
          vk::winit::VirtualKeyCode::LShift | vk::winit::VirtualKeyCode::RShift => self.shift = true,
          vk::winit::VirtualKeyCode::Tab => self.complete(self.shift),
          vk::winit::VirtualKeyCode::Up => self.history(false, None),
          vk::winit::VirtualKeyCode::Down => self.history(true, None),
          _ => (),
        },
        vk::winit::Event::WindowEvent {
          event:
            vk::winit::WindowEvent::KeyboardInput {
              input:
                vk::winit::KeyboardInput {
                  state: vk::winit::ElementState::Released,
                  virtual_keycode: Some(k),
                  ..
                },
              ..
            },
          ..
        } if self.show_term => match *k {
          vk::winit::VirtualKeyCode::LShift | vk::winit::VirtualKeyCode::RShift => self.shift = false,
          _ => (),
        },
        _ => (),
      }
    }

    // set focus after draw, so that we do not write the colon when enabling the terminal
    if let Some(f) = set_focus {
      self.term.focus(f);
    }

    exe
  }

  fn exec(&mut self, c: &str) -> Option<(Arc<dyn Command<S, C>>, Vec<String>)> {
    self.history.push(c.to_string());
    self.cmds.iter().find_map(|cmd| cmd.parse(c).map(|args| (cmd.clone(), args)))
  }

  fn get_completions(&self, input: &str) -> Option<Vec<arg::Completion>> {
    if self.cmds.iter().filter(|c| c.get_name().starts_with(&input)).count() > 1 {
      Some(self.cmds.iter().filter_map(|c| c.complete(&input)).flatten().collect::<Vec<_>>())
    } else {
      self.cmds.iter().find_map(|c| c.complete(&input))
    }
  }

  fn complete(&mut self, reverse: bool) {
    let input = self.term.get_input();
    let mut prefix = input[..self.prefix_len].to_string();
    match self.get_completions(&prefix) {
      Some(ref completions) if !completions.is_empty() => {
        match self.complete_index {
          CompleteIndex::Input => {
            let s = completions[0].get_completed();
            let lcp = completions.iter().skip(1).fold(s.len(), |_, c| {
              s.chars().zip(c.get_completed().chars()).take_while(|(a, b)| a == b).count()
            });
            if self.prefix_len == lcp {
              self.complete_index = match reverse {
                false => CompleteIndex::Index(0),
                true => CompleteIndex::Index(completions.len() - 1),
              };
              prefix = s[..lcp].to_string();
            } else {
              self.prefix_len = lcp;
              self.complete_index = CompleteIndex::Lcp;
              prefix = s[..lcp].to_string();
            }
          }
          CompleteIndex::Lcp => {
            self.complete_index = match reverse {
              false => CompleteIndex::Index(0),
              true => CompleteIndex::Index(completions.len() - 1),
            }
          }
          CompleteIndex::Index(i) => {
            let ci = i as i32
              + match reverse {
                false => 1,
                true => -1,
              };
            self.complete_index = if ci < 0 || ci >= completions.len() as i32 {
              CompleteIndex::Lcp
            } else {
              CompleteIndex::Index(ci as usize)
            };
          }
        }

        if let CompleteIndex::Index(ci) = self.complete_index {
          self.term.input_text(&completions[ci].get_completed());
        } else {
          self.term.input_text(&prefix);
        }
      }
      _ => (),
    }
  }

  fn history(&mut self, reverse: bool, find: Option<&str>) {
    let len = match find {
      Some(find) => self.history.iter().rev().filter(|s| s.find(find).is_some()).count(),
      _ => self.history.len(),
    };

    if !self.history.is_empty() {
      self.history_index = match self.history_index {
        HistoryIndex::Input => match reverse {
          false => HistoryIndex::Index(0),
          true => HistoryIndex::Index(len - 1),
        },
        HistoryIndex::Index(i) => {
          let i = i as i32
            + match reverse {
              false => 1,
              true => -1,
            };
          if i < 0 || i >= len as i32 {
            HistoryIndex::Input
          } else {
            HistoryIndex::Index(i as usize)
          }
        }
      };

      if let HistoryIndex::Index(i) = self.history_index {
        match find {
          Some(find) => {
            if let Some(s) = self.history.iter().rev().filter(|s| s.find(find).is_some()).nth(i) {
              self.term.input_text(&s);
            }
          }
          _ => self.term.input_text(&self.history[self.history.len() - 1 - i]),
        };
      } else {
        self.term.input_text("");
      }
    }
  }
}

pub struct Shell<S: Style, C> {
  shell: Arc<Mutex<ShellImpl<S, C>>>,
}

impl<S: Style, C> Clone for Shell<S, C> {
  fn clone(&self) -> Self {
    Self { shell: self.shell.clone() }
  }
}

unsafe impl<S: Style, C> Send for Shell<S, C> {}

impl<S: Style, C> Shell<S, C> {
  pub fn new(gui: &ImGui<S>) -> Self {
    let sh = Self {
      shell: Arc::new(Mutex::new(ShellImpl::new(gui))),
    };
    sh.add_command(Box::new(command::source::Cmd::new()));
    sh
  }

  fn add_command_inner(&self, cmd: Box<dyn Command<S, C>>) {
    let cmds = &mut self.shell.lock().unwrap().cmds;
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
    let cmds = &mut self.shell.lock().unwrap().cmds;
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
    let exe = { self.shell.lock().unwrap().exec(c) };
    if let Some((cmd, args)) = exe {
      cmd.run(args, term, context);
    }
  }

  pub fn get_commands(&self) -> Vec<Arc<dyn Command<S, C>>> {
    self.shell.lock().unwrap().cmds.clone()
  }

  pub fn get_history(&self) -> Vec<String> {
    self.shell.lock().unwrap().history.clone()
  }

  pub fn get_term(&self) -> TerminalWnd<S> {
    self.shell.lock().unwrap().term.clone()
  }

  pub fn get_show_term(&self) -> bool {
    self.shell.lock().unwrap().show_term
  }
}
