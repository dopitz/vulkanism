use super::ColumnLayout;
use super::Component;
use super::Layout;
use super::Window;
use crate::rect::Rect;
use crate::ImGui;
use crate::imgui::WindowSubmit;
use vk::cmd::commands::Scissor;
use vk::cmd::stream::*;

struct WindowComponent {
  scissor: Scissor,
  draw_mesh: usize,
  select_mesh: Option<usize>,
}

pub struct RootWindow {
  gui: ImGui,
  components: Vec<WindowComponent>,
}

impl RootWindow {
  pub fn new(gui: ImGui) -> Self {
    RootWindow {
      gui,
      components: Default::default(),
    }
  }

  pub fn push<T: Component>(&mut self, c: &T) {
    self.components.push(WindowComponent {
      scissor: Scissor::with_rect(c.get_rect().into()),
      draw_mesh: c.get_mesh(),
      select_mesh: None,
    });
  }

  pub fn begin_window(self) -> Window<ColumnLayout> {
    self.begin_layout(ColumnLayout::default())
  }
  pub fn begin_layout<T: Layout>(self, layout: T) -> Window<T> {
    let extent = self.gui.get_size();
    Window::new(self, layout).size(extent.width, extent.height)
  }

  pub fn end(self) -> WindowSubmit {
    self.gui.clone().end(self)
  }
}

impl StreamPushMut for RootWindow {
  fn enqueue_mut(&mut self, cs: CmdBuffer) -> CmdBuffer {
    let mut cs = self.gui.begin_draw(cs);

    //.push(&Scissor::with_rect(self.layout.get_rect().into()));

    let meshes = self.gui.get_meshes();
    for c in self.components.iter() {
      cs = cs.push(&c.scissor).push(&meshes.get(c.draw_mesh));
    }

    // TODO: Select pass

    cs = self.gui.end_draw(cs);

    self.components.clear();
    cs
  }
}
