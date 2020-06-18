use crate::component::Component;
use crate::component::Size;
use crate::rect::Rect;
use crate::select::SelectId;
use crate::style::Style;
use crate::window::Layout;
use crate::window::Screen;
use vk::cmd::commands::Scissor;
use vk::cmd::stream::*;
use vk::pass::MeshId;
use vk::winit;

#[derive(Clone, Copy, Debug)]
pub enum Type {
  HandleEvent,
  Draw,
}

pub struct Stream<'a, S: Style, R: std::fmt::Debug> {
  streamtype: Type,

  screen: Option<Screen<S>>,
  layout: Vec<Box<dyn Layout>>,
  focus: Option<&'a mut SelectId>,
  event: Option<&'a winit::event::Event<'a, i32>>,

  result: Option<R>,
}

impl<'a, S: Style> Stream<'a, S, ()> {
  pub fn new(s: Screen<S>, l: Box<dyn Layout>, f: &'a mut SelectId) -> Self {
    Self {
      streamtype: Type::Draw,
      screen: Some(s),
      layout: vec![l],
      focus: Some(f),
      event: None,
      result: None,
    }
  }
}

impl<'a, S: Style, R: std::fmt::Debug> Stream<'a, S, R> {
  pub fn draw(&mut self, mesh: MeshId, scissor: Scissor) {
    match self.screen.as_mut() {
      Some(s) => s.push_draw(mesh, scissor),
      None => (),
    }
  }
  pub fn select(&mut self, mesh: MeshId, scissor: Scissor) {
    match self.screen.as_mut() {
      Some(s) => s.push_select(mesh, scissor),
      None => (),
    }
  }

  pub fn get_selection(&mut self) -> Option<SelectId> {
    match self.screen.as_mut() {
      Some(s) => s.get_select_result(),
      None => None,
    }
  }

  pub fn push_layout(&mut self, l: Box<dyn Layout>) {
    self.layout.push(l);
  }
  pub fn pop_layout(&mut self) -> Box<dyn Layout>{
    self.layout.pop().unwrap()
  }

  pub fn get_type(&self) -> Type {
    self.streamtype
  }
  pub fn is_handle_event(&self) -> bool {
    match self.streamtype {
      Type::HandleEvent => true,
      Type::Draw => false,
    }
  }
  pub fn is_draw(&self) -> bool {
    !self.is_handle_event()
  }

  pub fn get_focus(&self) -> Option<&SelectId> {
    self.focus.as_ref().map(|f| &**f)
  }
  pub fn set_focus(&mut self, id: SelectId) {
    self.focus.as_mut().map(|f| **f = id);
  }

  pub fn get_event(&self) -> Option<&'a winit::event::Event<'a, i32>> {
    self.event.clone()
  }

  pub fn get_result(&self) -> Option<&R> {
    self.result.as_ref()
  }
  pub fn with_result<Rx: std::fmt::Debug>(self, r: Option<Rx>) -> Stream<'a, S, Rx> {
    Stream::<'a, S, Rx> {
      streamtype: self.streamtype,
      screen: self.screen,
      layout: self.layout,
      focus: self.focus,
      event: self.event,
      result: r,
    }
  }

  pub fn push<Rx: std::fmt::Debug, C: Component<S, Event = Rx>>(self, c: &mut C) -> Stream<'a, S, Rx> {
    c.enqueue(self)
  }
}

impl<'a, S: Style, R: std::fmt::Debug> Size for Stream<'a, S, R> {
  fn set_rect(&mut self, rect: Rect) {
    self.layout.last_mut().unwrap().set_rect(rect);
  }
  fn get_rect(&self) -> Rect {
    self.layout.last().unwrap().get_rect()
  }

  fn get_size_hint(&self) -> vkm::Vec2u {
    self.layout.last().unwrap().get_size_hint()
  }
}

impl<'a, S: Style, R: std::fmt::Debug> Layout for Stream<'a, S, R> {
  fn restart(&mut self) {
    self.layout.last_mut().unwrap().restart();
  }

  fn layout(&mut self, c: &mut dyn Size) -> Scissor {
    self.layout.last_mut().unwrap().layout(c)
  }

  fn get_scissor(&self, rect: Rect) -> Scissor {
    self.layout.last().unwrap().get_scissor(rect)
  }
}

impl<'a, S: Style, R: std::fmt::Debug> StreamPushMut for Stream<'a, S, R> {
  fn enqueue_mut(&mut self, cs: CmdBuffer) -> CmdBuffer {
    match self.screen.take() {
      Some(mut screen) => cs.push_mut(&mut screen),
      None => cs,
    }
  }
}
