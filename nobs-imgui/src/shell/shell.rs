use super::terminal::Event;
use super::*;
use crate::select::SelectId;
use crate::style::Style;
use crate::window::*;
use crate::ImGui;
use std::sync::Arc;
use std::sync::Mutex;

struct ShellImpl<S: Style, C> {
  term: Terminal<S>,

  cmds: Vec<Box<dyn Command<S, C>>>,

  show_term: bool,
  prefix_len: usize,
  complete_index: Option<usize>,
}

impl<S: Style, C> ShellImpl<S, C> {
  fn new(gui: &ImGui<S>) -> Self {
    Self {
      term: Terminal::new(gui),

      cmds: Default::default(),

      show_term: false,
      prefix_len: 0,
      complete_index: None,
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
      self.cmds.push(cmd);
    }
  }

  fn update<L: Layout>(&mut self, screen: &mut Screen<S>, layout: &mut L, focus: &mut SelectId, context: &mut C) {
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
          self.show_term = false;
          set_focus = Some(false);
        }
        _ => (),
      }
    }

    if self.show_term {
      match self.term.draw(screen, layout, focus) {
        Some(Event::TabComplete(shift)) => {
          let input = self.term.get_input();
          let prefix = input[..self.prefix_len].to_string();
          if let Some(completions) = self.get_completions(&prefix) {
            self.complete_index = match self.complete_index {
              None => match shift {
                false => Some(0),
                true => Some(completions.len() - 1),
              },
              Some(ci) => {
                let ci = ci as i32
                  + match shift {
                    false => 1,
                    true => -1,
                  };
                if ci < 0 || ci >= completions.len() as i32 {
                  None
                } else {
                  Some(ci as usize)
                }
              }
            };

            if let Some(&ci) = self.complete_index.as_ref() {
              self.term.input_text(&completions[ci].get_completed());
            } else {
              self.term.input_text(&prefix);
            }
          }
        }
        Some(Event::InputChanged) => {
          let input = self.term.get_input();
          self.prefix_len = input.len();
          self.complete_index = None;

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
          if let Some((cmd, args)) = self.cmds.iter().find_map(|cmd| cmd.parse(&input).map(|args| (cmd, args))) {
            cmd.run(args, &self.term, context);
          }
          self.prefix_len = 0;
          self.complete_index = None;
          self.term.quickfix_text("");
        }
        _ => (),
      }
    }

    // set focus after draw, so that we do not write the colon when enabling the terminal
    if let Some(f) = set_focus {
      self.term.focus(f);
    }
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

  pub fn add_command(&mut self, cmd: Box<dyn Command<S, C>>) {
    self.shell.lock().unwrap().add_command(cmd)
  }

  pub fn update<L: Layout>(&mut self, screen: &mut Screen<S>, layout: &mut L, focus: &mut SelectId, context: &mut C) {
    self.shell.lock().unwrap().update(screen, layout, focus, context);
  }

  pub fn get_term(&self) -> Terminal<S> {
    self.shell.lock().unwrap().term.clone()
  }

  pub fn get_show_term(&self) -> bool {
    self.shell.lock().unwrap().get_show_term()
  }
}
