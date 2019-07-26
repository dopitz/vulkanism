use super::ColumnLayout;
use super::Component;
use super::Layout;
use super::Window;
use crate::rect::Rect;
use crate::select::Query;
use crate::ImGui;
use vk::cmd::commands::Scissor;
use vk::cmd::stream::*;

struct WindowComponent {
  scissor: Scissor,
  draw_mesh: usize,
  select_mesh: Option<usize>,
}

pub struct RootWindow {
  gui: Option<ImGui>,
  components: Option<Vec<WindowComponent>>,
  query: Option<Query>,
}

impl RootWindow {
  pub fn new(gui: ImGui) -> Self {
    let query = Some(Query::new(gui.get_mem().clone()));
    RootWindow {
      gui: Some(gui),
      components: Some(Default::default()),
      query,
    }
  }
  pub fn from_cached(gui: ImGui, root: Self) -> Self {
    RootWindow {
      gui: Some(gui),
      components: root.components,
      query: root.query,
    }
  }

  pub fn push<T: Component>(&mut self, c: &T) {
    if let Some(components) = self.components.as_mut() {
      components.push(WindowComponent {
        scissor: Scissor::with_rect(c.get_rect().into()),
        draw_mesh: c.get_mesh(),
        select_mesh: c.get_select_mesh(),
      });
    }
  }

  pub fn begin_window(self) -> Window<ColumnLayout> {
    self.begin_layout(ColumnLayout::default())
  }
  pub fn begin_layout<T: Layout>(self, layout: T) -> Window<T> {
    let extent = self.gui.as_ref().unwrap().get_size();
    Window::new(self, layout).size(extent.width, extent.height)
  }

  pub fn get_select_result(&mut self) -> Option<u32> {
    self.query.as_mut().and_then(|q| q.get())
  }
}

impl StreamPushMut for RootWindow {
  fn enqueue_mut(&mut self, cs: CmdBuffer) -> CmdBuffer {
    let gui = self.gui.as_ref().unwrap();
    gui.get_select_rects().update();

    if let Some(mut components) = self.components.take() {
      // Draw actual ui elements
      let mut cs = gui.begin_draw(cs);
      let draw = gui.get_drawpass();
      for c in components.iter() {
        cs = cs.push(&c.scissor).push(&draw.get(c.draw_mesh));
      }
      cs = gui.end_draw(cs);

      // Select Query
      if let Some(mut query) = self.query.as_mut() {
        query.clear();
        for c in components.iter().filter(|c| c.select_mesh.is_some()) {
          query.push(c.select_mesh.unwrap(), Some(c.scissor))
        }
        cs = cs.push_mut(&mut gui.get_selectpass().push_query(query));
      }

      components.clear();
      gui.clone().end(Self {
        gui: None,
        components: Some(components),
        query: self.query.take(),
      });

      cs
    } else {
      cs
    }
  }
}
