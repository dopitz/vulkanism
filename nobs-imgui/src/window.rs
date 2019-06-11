use crate::rect::Rect;
use crate::ImGui;
use vk::builder::Buildable;
use vk::cmd::commands::Scissor;
use vk::cmd::commands::StreamPush;
use vk::cmd::commands::Viewport;
use vk::cmd::Stream;

pub trait Component {
  fn rect(&mut self, rect: Rect);

  fn get_mesh(&self) -> usize;
}

struct WinComp {
  scissor: Scissor,
  mesh: usize,
}

pub struct Window {
  gui: ImGui,
  rect: Rect,
  components: Vec<WinComp>,
}

impl Window {
  pub fn new(gui: ImGui) -> Self {
    Self {
      gui,
      rect: Default::default(),
      components: Default::default(),
    }
  }

  pub fn rect(mut self, rect: Rect) -> Self {
    self.rect = rect;
    self
  }
  pub fn size(mut self, w: u32, h: u32) -> Self {
    self.rect.size = vkm::Vec2::new(w, h);
    self
  }
  pub fn position(mut self, x: i32, y: i32) -> Self {
    self.rect.position = vkm::Vec2::new(x, y);
    self
  }

  pub fn push<T: Component>(mut self, c: &mut T) -> Self {
    // TODO: find next sub rect
    let rect = self.rect;

    c.rect(rect);
    self.components.push(WinComp {
      scissor: Scissor::with_rect(rect.to_vkrect()),
      mesh: c.get_mesh(),
    });

    self
  }
}

impl StreamPush for Window {
  fn enqueue(&self, cs: Stream) -> Stream {
    let mut cs = self.gui.begin(cs).push(&Scissor::with_rect(self.rect.to_vkrect()));

    let meshes = self.gui.get_meshes();
    for c in self.components.iter() {
      cs = cs.push(&c.scissor).push(&meshes.get(c.mesh));
    }

    self.gui.end(cs)
  }
}
