use crate::components::textbox::Event as TextboxEvent;
use crate::shell::arg::Completion;
use crate::shell::terminal::window::TerminalWnd;
use crate::shell::Context;
use crate::style::Style;
use crate::window::Screen;
use std::sync::Arc;
use std::sync::Mutex;
use vk::winit::Event;

#[derive(Clone, Copy)]
enum Index {
  Input,
  Prefix(usize),
  Complete(usize),
}
struct State {
  index: Index,
  completions: Option<Vec<Completion>>,
}

#[derive(Clone)]
pub struct Complete<S: Style> {
  window: TerminalWnd<S>,
  state: Arc<Mutex<State>>,
}

impl<S: Style> Complete<S> {
  pub fn new(window: TerminalWnd<S>) -> Self {
    Self {
      window,
      state: Arc::new(Mutex::new(State {
        index: Index::Input,
        completions: None,
      })),
    }
  }

  pub fn reset(&self) {
    let mut state = self.state.lock().unwrap();
    state.index = Index::Input;
    self.window.quickfix_text("");
  }

  pub fn handle_events<C: Context>(&self, screen: &mut Screen<S>, e: Option<TextboxEvent>, context: &C) {
    // handles the textbox event from the input box
    match e {
      Some(TextboxEvent::Enter(input)) => self.reset(),
      Some(TextboxEvent::Changed) => self.update_completions(context),
      _ => (),
    };

    for e in screen.get_events() {
      match e {
        vk::winit::Event::WindowEvent {
          event:
            vk::winit::WindowEvent::KeyboardInput {
              input:
                vk::winit::KeyboardInput {
                  state: vk::winit::ElementState::Pressed,
                  virtual_keycode: Some(vk::winit::VirtualKeyCode::Tab),
                  modifiers: vk::winit::ModifiersState { shift: reverse, .. },
                  ..
                },
              ..
            },
          ..
        } => self.next(*reverse),
        _ => (),
      }
    }

    //// execute the command
    //if let Some(s) = e {
    //  context.get_shell().exec(&s, context);
    //  self.input.history.push(s);
    //}
  }

  fn update_completions<C: Context>(&self, context: &C) {
    let input = self.window.get_input();

    // List of completions from command names, or parsed command arguments
    let cmds = context.get_shell().get_commands();

    println!("===========");
    for c in cmds.iter() {
      println!("{:?}", c.complete(&input));
      //println!("{:?}", c.parse(&input));
    }

    //if let Some(args) = cmds.iter().filter_map(|c| c.parse(&input)).next() {
    //  println!("{:?}", args);
    //}

    //let completions = if cmds.iter().filter(|c| c.get_commandname().starts_with(&input)).count() > 1 {
    //  Some(cmds.iter().filter_map(|c| c.complete(&input)).flatten().collect::<Vec<_>>())
    //} else {
    //  cmds.iter().find_map(|c| c.complete(&input))
    //};

    //// Update the quickfix window
    //if let Some(completions) = completions.as_ref() {
    //  let mut s = completions
    //    .iter()
    //    .fold(String::new(), |acc, c| format!("{}{}\n", acc, c.get_preview()));
    //  s = format!("{}{}", s, "-------------");
    //  self.window.quickfix_text(&s);
    //} else {
    //  self.window.quickfix_text("");
    //}

    //self.state.lock().unwrap().completions = completions;
  }

  fn next(&self, reverse: bool) {
    let input = self.window.get_input();

    let mut state = self.state.lock().unwrap();
    let mut index = state.index;

    match state.completions.as_ref() {
      Some(ref completions) if !completions.is_empty() => {
        match index {
          Index::Input => {
            let s = completions[0].get_completed();
            let lcp = completions.iter().skip(1).fold(s.len(), |_, c| {
              s.chars().zip(c.get_completed().chars()).take_while(|(a, b)| a == b).count()
            });
            index = match reverse {
              false => Index::Complete(0),
              true => Index::Complete(completions.len() - 1),
            };
          }
          Index::Prefix(i) => {
            match reverse {
              false => index = Index::Input,
              true => index = Index::Complete(0),
            };
          }
          Index::Complete(i) => {
            let d = match reverse {
              false => 1,
              true => -1,
            };
            let ci = i as i32 + d;
            index = if ci < 0 || ci >= completions.len() as i32 {
              Index::Prefix(input.len())
            } else {
              Index::Complete(ci as usize)
            };
          }
        }

        match index {
          Index::Complete(i) => self.window.input_text(&completions[i].get_completed()),
          Index::Prefix(i) => self.window.input_text(&input[..i]),
          _ => (),
        }
      }
      _ => (),
    }

    state.index = index;
  }
}
