use crate::select::Query;
use crate::select::SelectId;
use crate::style::Style;
use crate::ImGui;
use crate::rect::Rect;
use vk::cmd::commands::RenderpassBegin;
use vk::cmd::commands::RenderpassEnd;
use vk::cmd::commands::Scissor;
use vk::cmd::stream::*;
use vk::pass::MeshId;

struct WindowComponent {
  scissor: Scissor,
  mesh: MeshId,
}

/// Cache for gui components and selection query
///
/// The Screen capsulates gui rendering. This is done by getting the Screen from the [ImGui](../struct.ImGui.html) with [begin()](../struct.ImGui.html#method.begin()).
/// From here we can create logical [Windows](struct.Window.html) to layout and size components on the Screen.
///
/// The Screen lets one [push](struct.Screen.html#method.push) gui components. This will record the component's draw and selection mesh ids and scissor rect.
/// Gui components have to implement the [draw](trait.Component.html#method.draw) method and should be pushed to the respective [Window](struct.Window.html) there.
/// This allows to also handle user interaction with the component through it's return value.
///
/// After subitting all components to the screen, it may be enqueed to a command buffer.
///
/// Screen creation and lifetime management is handled by [ImGui](../struct.ImGui.html).
/// Pushing the Screen into a command buffer will automatically yield the Screen to the gui again, so that buffers and the select query can be reused in the next frame
pub struct Screen<S: Style> {
  gui: Option<ImGui<S>>,
  size: vk::Extent2D,
  image: vk::Image,
  draw_begin: RenderpassBegin,
  draw_end: RenderpassEnd,
  components: Option<Vec<WindowComponent>>,
  query: Option<[Query; 2]>,
}

impl<S: Style> Screen<S> {
  /// Creates a new Screen
  ///
  /// [ImGui](../struct.ImGui.html) contains a cached Screen that will be reused.
  /// Screen creation is then done with [ImGui::begin](../struct.ImGui.html#method.begin).
  /// This automatically decides if a new Screen needs to be created with [new](struct.Screen.html#method.new) or [from_cached](struct.Screen.html#method.from_cached)
  ///
  /// # Arguments
  /// * `gui` - Gui object for which the Screen is created
  /// * `size` - Current size of the gui's render target
  /// * `image` - Color render target of the gui (needed to enqueue a barrier before the draw pass is started)
  /// * `draw_begin` - Command to start the draw pass to render gui components
  /// * `draw_end` - Command to finish the draw pass to render gui components
  pub fn new(gui: ImGui<S>, size: vk::Extent2D, image: vk::Image, draw_begin: RenderpassBegin, draw_end: RenderpassEnd) -> Self {
    let query = Some([Query::new(gui.get_mem().clone()), Query::new(gui.get_mem().clone())]);
    Screen {
      gui: Some(gui),
      size,
      image,
      draw_begin,
      draw_end,
      events: Some(Default::default()),
      components: Some(Default::default()),
      query,
    }
  }
  /// Creates a Screen from a cached one
  ///
  /// See [new](struct.Screen.html#method.new) for details on Screen creation with [ImGui](../struct.ImGui.html).
  ///
  /// Creates a Screen while reusing heap allocated objects from `src`.
  ///
  /// # Arguments
  /// * `gui` - Gui object for which the Screen is created
  /// * `size` - Current size of the gui's render target
  /// * `image` - Color render target of the gui (needed to enqueue a barrier before the draw pass is started)
  /// * `draw_begin` - Command to start the draw pass to render gui components
  /// * `draw_end` - Command to finish the draw pass to render gui components
  /// * `scr` - Screen from which heap allocated objects are retrieved
  pub fn from_cached(
    gui: ImGui<S>,
    size: vk::Extent2D,
    image: vk::Image,
    draw_begin: RenderpassBegin,
    draw_end: RenderpassEnd,
    scr: Self,
  ) -> Self {
    Screen {
      gui: Some(gui),
      size,
      image,
      draw_begin,
      draw_end,
      events: scr.events,
      components: scr.components,
      query: scr.query,
    }
  }

  /// Records a mesh for drawing
  ///
  /// We first collect meshes in the `Screen` and then build the command buffer when the `Screen` is ready for rendering.
  ///
  /// # Arguments
  /// `mesh` - the MeshId to be drawn, the mesh id id MUST be retrieved and managed by the gui's [DrawPass](../struct.ImGui.html#method.get_drawpass)
  /// `scissor` - the scissor rect for the mesh
  pub fn push_draw(&mut self, mesh: MeshId, scissor: Scissor) {
    if let Some(components) = self.components.as_mut() {
      components.push(WindowComponent { scissor, mesh });
    }
  }
  /// Records a mesh for object selection
  ///
  /// Pushes the mesh into the current queue for object selection.
  ///
  /// # Arguments
  /// `mesh` - the MeshId to be drawn, the mesh id id MUST be retrieved and managed by the gui's [SelectPass](../select/struct.Select.html)
  /// `scissor` - the scissor rect for the mesh
  pub fn push_select(&mut self, mesh: MeshId, scissor: Scissor) {
    if let Some(query) = self.query.as_mut() {
      query[0].push(mesh, Some(scissor));
    }
  }

  /// Gets the selection result of the last query
  ///
  /// Object selection for the complete gui pass is done within one [Query](../select/struct.Query.html).
  /// Screen manages the queries. Two queries are used in a ping pong pattern so that we can still retrieve the query result while building a new query.
  ///
  /// # Returns
  /// The id of the selection, `None` if no valid object in the [Selection](../select/struct.Select.html) was selected or the query was not executed.
  pub fn get_select_result(&mut self) -> Option<SelectId> {
    self.query.as_mut().and_then(|q| q[1].get())
  }

  /// Get the size of the Screen
  ///
  /// # Returns
  /// The size of the screen as [Rect](struct.Rect.html).
  /// The top left of the screen (`rect.position`) is set to (0,0).
  pub fn get_rect(&self) -> Rect {
    Rect::new(vec2!(0), vec2!(self.size.width, self.size.height))
  }
}

impl<S: Style> StreamPushMut for Screen<S> {
  fn enqueue_mut(&mut self, cs: CmdBuffer) -> CmdBuffer {
    let gui = self.gui.as_ref().unwrap();

    if let Some(mut components) = self.components.take() {
      // Draw actual ui elements
      let mut cs = cs
        .push(&vk::cmd::commands::ImageBarrier::to_color_attachment(self.image))
        .push(&self.draw_begin)
        .push(&vk::cmd::commands::Viewport::with_extent(self.size))
        .push(&vk::cmd::commands::Scissor::with_extent(self.size));

      let draw = gui.get_drawpass();
      for c in components.iter() {
        cs = cs.push(&c.scissor).push(&draw.get(c.mesh));
      }

      cs = cs.push(&self.draw_end);

      // Select Query
      if let Some(query) = self.query.as_mut() {
        //query[0].clear();
        //for c in components.iter().filter(|c| c.select_mesh.is_some()) {
        //  query[0].push(c.select_mesh.unwrap(), Some(c.scissor))
        //}
        cs = cs.push_mut(&mut gui.select.push_query(&mut query[0]));
        // only clears meshes, leaves result untouched...
        query[0].clear();
        // swap queries
        query.swap(0, 1);
      }

      let mut events = self.events.take().unwrap();
      events.clear();
      components.clear();
      gui.clone().end(Self {
        gui: None,
        size: self.size,
        image: self.image,
        draw_begin: self.draw_begin,
        draw_end: self.draw_end,
        events: Some(events),
        components: Some(components),
        query: self.query.take(),
      });

      cs
    } else {
      cs
    }
  }
}
