use super::Layout;
use super::Window;
use crate::rect::Rect;
use crate::select::SelectId;
use vk::pass::MeshId;

/// Basic trait for renderable and selectable gui components
///
/// Enforces functions for resizing and drawing components.
pub trait Component {
  /// Sets the size and position of the component
  fn rect(&mut self, rect: Rect) -> &mut Self;
  /// Gets the current size and position of the component
  fn get_rect(&self) -> Rect;

  /// Gets the ideal size of the component
  fn get_size_hint(&self) -> vkm::Vec2u;

  /// Event type that can be used to handle user interaction when the component is [drawn](trait.Component.html#method.draw)
  type Event;
  /// Draws the component and returns an Event for handling user interaction
  ///
  /// The component is added to the Screen referenced by `wnd`.
  /// The window is used to resize and set the position of the component with [rect](trait.Component.html#method.rect).
  fn draw<T: Layout>(&mut self, wnd: &mut Window<T>, focus: &mut SelectId) -> Option<Self::Event>;
}
