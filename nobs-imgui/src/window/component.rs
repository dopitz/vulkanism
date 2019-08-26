use super::Screen;
use super::Layout;
use crate::rect::Rect;
use crate::select::SelectId;
use crate::style::Style;

/// Basic trait for renderable and selectable gui components
///
/// Enforces functions for resizing and drawing components.
pub trait Component<S: Style> {
  /// Sets the size and position of the component
  fn rect(&mut self, rect: Rect) -> &mut Self;
  /// Gets the current size and position of the component
  fn get_rect(&self) -> Rect;

  /// Gets the ideal size of the component
  ///
  /// A [Layout](struct.Layout.html) may use this size as a guide and will (implementation dependent try) to adhere by the component's ideal size.
  fn get_size_hint(&self) -> vkm::Vec2u;

  /// Event type that can be used to handle user interaction when the component is [drawn](trait.Component.html#method.draw)
  type Event : std::fmt::Debug;
  /// Draws the component and returns an Event for handling user interaction
  ///
  /// This function may sereve a double purpose
  ///  1. Draw the component (normal + object selection)
  ///  2. Handle events
  ///
  /// # Arguments
  ///  * `screen` - Screen to draw on
  ///  * `layout` - Layout to position and resize the component
  ///  * `focus` - The currently focused component (should be ignored when this does not match the select id of self)
  ///
  /// # Returns
  ///  * `None` - if no event was handles
  ///  * `Some(Event)` - if an event was handled
  fn draw<L: Layout>(&mut self, screen: &mut Screen<S>, layout: &mut L, focus: &mut SelectId) -> Option<Self::Event>;
}
