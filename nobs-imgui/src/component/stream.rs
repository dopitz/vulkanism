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

pub struct StreamCache<S: Style> {
  screen: Option<Screen<S>>,
  focus: Option<SelectId>,
  layout: Option<Vec<Box<dyn Layout>>>,
}

impl<S: Style> StreamCache<S> {
  pub fn new() -> Self {
    Self {
      screen: None,
      layout: Some(Vec::with_capacity(10)),
      focus: Some(SelectId::invalid()),
    }
  }

  pub fn into_stream<'a>(&mut self, event: Option<&'a winit::event::Event<i32>>) -> Stream<'a, S, ()> {
    Stream {
      streamtype: Type::HandleEvent,
      screen: self.screen.take().unwrap(),
      layout: self.layout.take().unwrap(),
      focus: self.focus.take().unwrap(),
      event,
      result: None,
    }
  }

  pub fn recover<'a, R: std::fmt::Debug>(&mut self, s: Stream<'a, S, R>) {
    self.screen = Some(s.screen);
    self.layout = Some(s.layout);
    self.focus = Some(s.focus);
  }
}

pub struct Stream<'a, S: Style, R: std::fmt::Debug> {
  streamtype: Type,

  screen: Screen<S>,
  layout: Vec<Box<dyn Layout>>,
  focus: SelectId,
  event: Option<&'a winit::event::Event<'a, i32>>,

  result: Option<R>,
}

impl<'a, S: Style, R: std::fmt::Debug> Stream<'a, S, R> {
  pub fn draw(&mut self, mesh: MeshId, scissor: Scissor) {
    match self.streamtype {
      Type::Draw => self.screen.push_draw(mesh, scissor),
      Type::HandleEvent => (),
    }
  }
  pub fn select(&mut self, mesh: MeshId, scissor: Scissor) {
    match self.streamtype {
      Type::Draw => self.screen.push_select(mesh, scissor),
      Type::HandleEvent => (),
    }
  }

  pub fn get_selection(&mut self) -> Option<SelectId> {
    self.screen.get_select_result()
  }
  pub fn is_selected(&mut self, id: SelectId) -> bool {
    self.get_selection().filter(|s| *s == id && id != SelectId::invalid()).is_some()
  }

  pub fn push_layout(&mut self, l: Box<dyn Layout>) {
    self.layout.push(l);
  }
  pub fn pop_layout(&mut self) -> Box<dyn Layout> {
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
    Some(&self.focus)
  }
  pub fn set_focus(&mut self, id: SelectId) {
    self.focus = id;
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

  pub fn push_if<C: Component<S, Event = R>>(self, b: bool, c: &mut C) -> Stream<'a, S, R> {
    if b {
      self.push(c)
    } else {
      self
    }
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
    match self.streamtype {
      Type::Draw => cs.push_mut(&mut self.screen),
      Type::HandleEvent => cs,
    }
  }
}
