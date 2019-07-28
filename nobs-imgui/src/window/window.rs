use super::Component;
use super::ColumnLayout;
use super::Layout;
use super::Screen;
use crate::rect::Rect;

/// Window used to set position and size of gui components
///
/// The Window defines a region on the screen on which components are draw
pub struct Window<'a, T: Layout> {
  scr: &'a mut Screen,
  layout: T,
}

impl<'a> Window<'a, ColumnLayout> {
  /// Creates a new window on the spcified [Screen](struct.Screen.html).
  ///
  /// The windows layout will be a [ColumnLayout](struct.ColumnLayout.html)
  pub fn new(scr: &'a mut Screen) -> Self {
    Self::with_layout(scr, ColumnLayout::default())
  }
}

impl<'a, T: Layout> Window<'a, T> {
  /// Creates a new window on the spcified [Screen](struct.Screen.html).
  ///
  /// Sets the specified layout for the window
  pub fn with_layout(scr: &'a mut Screen, layout: T) -> Self {
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

  /// Add the spcified component to the window
  ///
  /// The component will be positioned and resized according to the Window's [Layout](trait.Layout.html)
  pub fn push<C: Component>(&mut self, c: &mut C) {
    self.layout.push(c);
    self.scr.push(c);
  }

  /// Get the selection result of the last object query
  ///
  /// See [get_select_result](struct.Screen.html#method.get_select_result).
  pub fn get_select_result(&mut self) -> Option<u32> {
    self.scr.get_select_result()
  }

  /// Gets the list of events since last time [ImGui::handle_events](../struct.Imgui.html#method.handle_events) was called.
  pub fn get_events(&'a self) -> &'a[vk::winit::Event] {
    self.scr.get_events()
  }
}
