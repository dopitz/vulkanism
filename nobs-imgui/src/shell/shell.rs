use super::terminal::Event;
use super::*;
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
  Complete(usize),
}

struct ShellImpl<S: Style, C> {
  term: Terminal<S>,

  cmds: Vec<Arc<dyn Command<S, C>>>,

  show_term: bool,
  prefix_len: usize,
  //complete_index: Option<usize>,
  complete_index: CompleteIndex,
}

impl<S: Style, C> ShellImpl<S, C> {
  fn new(gui: &ImGui<S>) -> Self {
    Self {
      term: Terminal::new(gui),

      cmds: Default::default(),

      show_term: false,
      prefix_len: 0,
      complete_index: CompleteIndex::Input,
    }
  }

  fn add_command(&mut self, cmd: Box<dyn Command<S, C>>) {
    let name = cmd.get_name();
    if let Some(c) = self
      .cmds
      .iter()
      .find(|c| c.get_name().starts_with(name) || name.starts_with(c.get_name()))
    {
      println!("Command can not be added. Name conflict:\n{}\n{}", name, c.get_name());
    } else {
      self.cmds.push(cmd.into());
    }
  }

  fn drop_command(&mut self, name: &str) {
    if let Some(p) = self.cmds.iter().position(|c| c.get_name() == name) {
      self.cmds.remove(p);
    }
  }

  fn update<L: Layout>(
    &mut self,
    screen: &mut Screen<S>,
    layout: &mut L,
    focus: &mut SelectId,
  ) -> Option<(Arc<dyn Command<S, C>>, Vec<String>)> {
    let mut set_focus = None;
    for e in screen.get_events() {
      match e {
        vk::winit::Event::WindowEvent {
          event: vk::winit::WindowEvent::ReceivedCharacter(':'),
          ..
        } if !self.show_term => {
          self.show_term = true;
          set_focus = Some(true);
        }
        vk::winit::Event::WindowEvent {
          event:
            vk::winit::WindowEvent::KeyboardInput {
              input:
                vk::winit::KeyboardInput {
                  state: vk::winit::ElementState::Pressed,
                  virtual_keycode: Some(vk::winit::VirtualKeyCode::Escape),
                  ..
                },
              ..
            },
          ..
        } if self.show_term => {
          if self.term.get_input().is_empty() {
            self.show_term = false;
            set_focus = Some(false);
          } else {
            self.term.input_text("");
            self.term.quickfix_text("");
          }
        }
        _ => (),
      }
    }

    let mut exe = None;
    if self.show_term {
      match self.term.draw(screen, layout, focus) {
        Some(Event::TabComplete(shift)) => {
          let input = self.term.get_input();
          let mut prefix = input[..self.prefix_len].to_string();
          match self.get_completions(&prefix) {
            Some(ref completions) if !completions.is_empty() => {
              match self.complete_index {
                CompleteIndex::Input => {
                  let s = completions[0].get_completed();
                  let lcp = completions.iter().skip(1).fold(s.len(), |_, c| {
                    s
                      .chars()
                      .zip(c.get_completed().chars())
                      .take_while(|(a, b)| a == b)
                      .count()
                  });
                  self.prefix_len = lcp;
                  self.complete_index = CompleteIndex::Lcp;
                  prefix = s[..lcp].to_string();
                }
                CompleteIndex::Lcp => {
                  self.complete_index = match shift {
                    false => CompleteIndex::Complete(0),
                    true => CompleteIndex::Complete(completions.len() - 1),
                  }
                }
                CompleteIndex::Complete(i) => {
                  let ci = i as i32
                    + match shift {
                      false => 1,
                      true => -1,
                    };
                  self.complete_index = if ci < 0 || ci >= completions.len() as i32 {
                    CompleteIndex::Lcp
                  } else {
                    CompleteIndex::Complete(ci as usize)
                  };
                }
              }

              if let CompleteIndex::Complete(ci) = self.complete_index {
                self.term.input_text(&completions[ci].get_completed());
              } else {
                self.term.input_text(&prefix);
              }
            }
            _ => (),
          }
        }
        Some(Event::InputChanged) => {
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
        Some(Event::InputSubmit(input)) => {
          exe = self.exec(&input);
          self.prefix_len = 0;
          self.complete_index = CompleteIndex::Input;
          self.term.quickfix_text("");
        }
        _ => (),
      }
    }

    // set focus after draw, so that we do not write the colon when enabling the terminal
    if let Some(f) = set_focus {
      self.term.focus(f);
    }

    exe
  }

  fn exec(&self, c: &str) -> Option<(Arc<dyn Command<S, C>>, Vec<String>)> {
    self.cmds.iter().find_map(|cmd| cmd.parse(c).map(|args| (cmd.clone(), args)))
  }

  fn get_show_term(&self) -> bool {
    self.show_term
  }

  fn get_completions(&self, input: &str) -> Option<Vec<arg::Completion>> {
    if self.cmds.iter().filter(|c| c.get_name().starts_with(&input)).count() > 1 {
      Some(self.cmds.iter().filter_map(|c| c.complete(&input)).flatten().collect::<Vec<_>>())
    } else {
      self.cmds.iter().find_map(|c| c.complete(&input))
    }
  }
}

#[derive(Clone)]
pub struct Shell<S: Style, C> {
  shell: Arc<Mutex<ShellImpl<S, C>>>,
}

impl<S: Style, C> Shell<S, C> {
  pub fn new(gui: &ImGui<S>) -> Self {
    Self {
      shell: Arc::new(Mutex::new(ShellImpl::new(gui))),
    }
  }

  pub fn add_command(&self, cmd: Box<dyn Command<S, C>>) {
    self.shell.lock().unwrap().add_command(cmd);
    self.update_help();
  }

  pub fn drop_command(&self, name: &str) {
    self.shell.lock().unwrap().drop_command(name);
    self.update_help();
  }

  fn update_help(&self) {
    let mut shell = self.shell.lock().unwrap();
    shell.drop_command(command::help::Cmd::get_name());
    let help = command::help::Cmd::new::<S, C>(&shell.cmds);
    shell.add_command(Box::new(help));
  }

  pub fn update<L: Layout>(&self, screen: &mut Screen<S>, layout: &mut L, focus: &mut SelectId, context: &mut C) {
    let exe = { self.shell.lock().unwrap().update(screen, layout, focus) };
    if let Some((cmd, args)) = exe {
      cmd.run(args, Self { shell: self.shell.clone() }, context);
    }
  }

  pub fn exec(&self, c: &str, context: &mut C) {
    let exe = { self.shell.lock().unwrap().exec(c) };
    if let Some((cmd, args)) = exe {
      cmd.run(args, Self { shell: self.shell.clone() }, context);
    }
  }

  pub fn get_commands(&self) -> Vec<Arc<dyn Command<S, C>>> {
    self.shell.lock().unwrap().cmds.clone()
  }

  pub fn get_term(&self) -> Terminal<S> {
    self.shell.lock().unwrap().term.clone()
  }

  pub fn get_show_term(&self) -> bool {
    self.shell.lock().unwrap().get_show_term()
  }
}
