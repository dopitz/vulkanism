use super::ColumnLayout;
use super::Component;
use super::Layout;
use super::Screen;
use crate::rect::Rect;
use crate::style::Style;
use crate::select::SelectId;
use vk::cmd::commands::Scissor;
use vk::pass::MeshId;

/// Window used to set position and size of gui components
///
/// The Window defines a region on the screen on which components are draw
/// It is basically a builder pattern around a [Layout](struct.Layout.html) and [Screen](struct.Streen.html)
pub struct Window<'a, L: Layout, S: Style> {
  scr: &'a mut Screen<S>,
  layout: L,
}

impl<'a, S: Style> Window<'a, ColumnLayout, S> {
  /// Creates a new window on the spcified [Screen](struct.Screen.html).
  ///
  /// The windows layout will be a [ColumnLayout](struct.ColumnLayout.html)
  pub fn new(scr: &'a mut Screen<S>) -> Self {
    Self::with_layout(scr, ColumnLayout::default())
  }
}

impl<'a, L: Layout, S: Style> Window<'a, L, S> {
  /// Creates a new window on the spcified [Screen](struct.Screen.html).
  ///
  /// Sets the specified layout for the window
  pub fn with_layout(scr: &'a mut Screen<S>, layout: L) -> Self {
    Self { scr, layout }
  }

  /// Sets size and position of the Window in pixel coordinates
  pub fn rect(mut self, rect: Rect) -> Self {
    self.layout.reset(rect);
    self
  }
  /// Sets the size of the Window in pixel coordinates
  pub fn size(mut self, w: u32, h: u32) -> Self {
    self.layout.reset(Rect::new(self.layout.get_rect().position, vkm::Vec2::new(w, h)));
    self
  }
  /// Sets the position of the Window in pixel coordinates
  pub fn position(mut self, x: i32, y: i32) -> Self {
    self.layout.reset(Rect::new(vkm::Vec2::new(x, y), self.layout.get_rect().size));
    self
  }

  /// Resizes component according to the window`s layout
  ///
  /// See [Layout](struct.Layout.html)
  pub fn apply_layout<C: Component<S>>(&mut self, c: &mut C) -> Scissor {
    self.layout.apply(c)
  }
  /// Records a mesh for drawing
  ///
  /// See [Screen](struct.Screen.html)
  pub fn push_draw(&mut self, mesh: MeshId, scissor: Scissor) {
    self.scr.push_draw(mesh, scissor);
  }
  /// Records a mesh for drawing
  ///
  /// See [Screen](struct.Screen.html)
  pub fn push_select(&mut self, mesh: MeshId, scissor: Scissor) {
    self.scr.push_select(mesh, scissor);
  }

  /// Get the selection result of the last object query
  ///
  /// See [get_select_result](struct.Screen.html#method.get_select_result).
  pub fn get_select_result(&mut self) -> Option<SelectId> {
    self.scr.get_select_result()
  }

  /// Gets the list of events since last time [ImGui::handle_events](../struct.Imgui.html#method.handle_events) was called.
  pub fn get_events(&'a self) -> &'a [vk::winit::Event] {
    self.scr.get_events()
  }
}
