use crate::component::Component;
use crate::component::Size;
use crate::rect::Rect;
use crate::select::Query;
use crate::select::SelectId;
use crate::style::Style;
use crate::window::FloatLayout;
use crate::window::Layout;
use crate::ImGui;
use vk::cmd::commands::RenderpassBegin;
use vk::cmd::commands::RenderpassEnd;
use vk::cmd::commands::Scissor;
use vk::cmd::stream::*;
use vk::pass::MeshId;
use vk::winit;

#[derive(Debug)]
struct DrawInfo {
  mesh: MeshId,
  rect: Scissor,
}

#[derive(Clone, Copy, Debug)]
pub enum Type {
  HandleEvent,
  Draw,
}

pub struct StreamCache {
  components: Option<Vec<DrawInfo>>,
  query: Option<[Query; 2]>,
  layout: Option<Vec<Box<dyn Layout>>>,
  focus: Option<SelectId>,
}

impl StreamCache {
  pub fn new(mem: &vk::mem::Mem) -> Self {
    Self {
      components: Some(Vec::with_capacity(100)),
      query: Some([Query::new(mem.clone()), Query::new(mem.clone())]),
      layout: Some(Vec::with_capacity(10)),
      focus: Some(SelectId::invalid()),
    }
  }

  pub fn into_stream<'a, S: Style>(
    &mut self,
    gui: ImGui<S>,
    size: vk::Extent2D,
    image: vk::Image,
    pass_begin: RenderpassBegin,
    pass_end: RenderpassEnd,
    event: Option<&'a winit::event::Event<i32>>,
  ) -> Stream<'a, S, ()> {
    let components = self.components.take().unwrap();
    let query = self.query.take().unwrap();
    let mut layout = self.layout.take().unwrap();
    layout.push(Box::new(FloatLayout::from(Rect::new(vec2!(0), vec2!(size.width, size.height)))));
    let focus = self.focus.take().unwrap();

    Stream {
      gui,

      streamtype: match event {
        Some(_) => Type::HandleEvent,
        None => Type::Draw,
      },

      size,
      image,
      pass_begin,
      pass_end,
      components,
      query,

      layout,
      focus,
      event,
      result: None,
    }
  }

  pub fn recover<'a, S: Style, R: std::fmt::Debug>(&mut self, s: Stream<'a, S, R>) {
    self.components = Some(s.components);
    self.query = Some(s.query);
    self.layout = Some(s.layout);
    self.focus = Some(s.focus);
  }
}

pub struct Stream<'a, S: Style, R: std::fmt::Debug> {
  gui: ImGui<S>,

  streamtype: Type,

  size: vk::Extent2D,
  image: vk::Image,
  pass_begin: RenderpassBegin,
  pass_end: RenderpassEnd,
  components: Vec<DrawInfo>,
  query: [Query; 2],

  layout: Vec<Box<dyn Layout>>,
  focus: SelectId,
  event: Option<&'a winit::event::Event<'a, i32>>,

  result: Option<R>,
}

impl<'a, S: Style, R: std::fmt::Debug> Stream<'a, S, R> {
  pub fn draw(&mut self, mesh: MeshId, rect: Scissor) {
    match self.streamtype {
      Type::Draw => self.components.push(DrawInfo { mesh, rect }),
      Type::HandleEvent => (),
    }
  }
  pub fn select(&mut self, mesh: MeshId, rect: Scissor) {
    match self.streamtype {
      Type::Draw => self.query[0].push(mesh, Some(rect)),
      Type::HandleEvent => (),
    }
  }

  pub fn get_selection(&mut self) -> Option<SelectId> {
    self.query[1].get()
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
    self.map_result(|_| r)
  }
  pub fn map_result<Rx: std::fmt::Debug, F: FnOnce(Option<R>) -> Option<Rx>>(self, r: F) -> Stream<'a, S, Rx> {
    let result = r(self.result);
    Stream::<'a, S, Rx> {
      gui: self.gui,

      streamtype: self.streamtype,

      size: self.size,
      image: self.image,
      pass_begin: self.pass_begin,
      pass_end: self.pass_end,
      components: self.components,
      query: self.query,

      layout: self.layout,
      focus: self.focus,
      event: self.event,
      result,
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
    use vk::cmd::stream::Stream;
    match self.streamtype {
      Type::Draw => {
        // Draw actual ui elements
        let mut cs = cs
          .push(&vk::cmd::commands::ImageBarrier::to_color_attachment(self.image))
          .push(&self.pass_begin)
          .push(&vk::cmd::commands::Viewport::with_extent(self.size))
          .push(&vk::cmd::commands::Scissor::with_extent(self.size));

        let draw = self.gui.get_drawpass();
        for c in self.components.iter() {
          cs = cs.push(&c.rect).push(&draw.get(c.mesh));
        }

        cs = cs.push(&self.pass_end);
        cs = cs.push_mut(&mut self.gui.select.push_query(&mut self.query[0]));
        // only clears meshes in q[0], 
        // reset result in q[1]
        self.query[0].clear();
        self.query[1].reset();
        // swap queries
        self.query.swap(0, 1);

        self.components.clear();
        self.layout.clear();
        cs
      }
      Type::HandleEvent => cs,
    }
  }
}
