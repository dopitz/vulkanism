use crate::components::textbox::Event as TextboxEvent;
use crate::shell::command::args;
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
  input: String,
  completions: Vec<args::Completion>,
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
        input: String::new(),
        completions: Vec::new(),
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
    let mut state = self.state.lock().unwrap();
    let input = self.window.get_input();

    // List of completions from command names, or parsed command arguments
    let cmds = context.get_shell().get_commands();

    state.input = input.clone();
    state.completions.clear();
    for c in cmds.iter() {
      c.parse(&input, Some(&mut state.completions));
    }

    println!("{:?}", state.completions);

    if !state.completions.is_empty() {
      let mut s = state.completions.first().unwrap().completed.to_string();
      for c in state.completions.iter().skip(1) {
        s = format!("{}\n{}", s, c.completed);
      }
      println!("{}", s);
      self.window.quickfix_text(&format!("===============\n{}", s));
    } else {
      self.window.quickfix_text("");
    }
  }

  fn next(&self, reverse: bool) {
    //  let input = self.window.get_input();

    //  let mut state = self.state.lock().unwrap();
    //  let mut index = state.index;

    //  if !state.completions.is_empty() {
    //    match index {
    //      Index::Input => {
    //        let s = state.completions[0].get_completed();
    //        let lcp = state.completions.iter().skip(1).fold(s.len(), |_, c| {
    //          s.chars().zip(c.get_completed().chars()).take_while(|(a, b)| a == b).count()
    //        });
    //        index = match reverse {
    //          false => Index::Complete(0),
    //          true => Index::Complete(state.completions.len() - 1),
    //        };
    //      }
    //      Index::Prefix(i) => {
    //        match reverse {
    //          false => index = Index::Input,
    //          true => index = Index::Complete(0),
    //        };
    //      }
    //      Index::Complete(i) => {
    //        let d = match reverse {
    //          false => 1,
    //          true => -1,
    //        };
    //        let ci = i as i32 + d;
    //        index = if ci < 0 || ci >= state.completions.len() as i32 {
    //          Index::Prefix(input.len())
    //        } else {
    //          Index::Complete(ci as usize)
    //        };
    //      }
    //    }

    //    match index {
    //      Index::Complete(i) => self.window.input_text(&state.completions[i].get_completed()),
    //      Index::Prefix(i) => self.window.input_text(&input[..i]),
    //      _ => (),
    //    }
    //  }

    //  state.index = index;
  }
}
