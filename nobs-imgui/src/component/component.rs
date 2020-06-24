use crate::component::Size;
use crate::component::Stream;
use crate::select::SelectId;
use crate::style::Style;
use crate::window::Layout;
use vk::winit;

///// Basic trait for renderable and selectable gui components
/////
///// Enforces functions for resizing and drawing components.
//pub trait Component<S: Style> : Size {
//  /// Event type that can be used to handle user interaction when the component is [drawn](trait.Component.html#method.draw)
//  type Event: std::fmt::Debug;
//  /// Draws the component and returns an Event for handling user interaction
//  ///
//  /// This function may sereve a double purpose
//  ///  1. Draw the component (normal + object selection)
//  ///  2. Handle events
//  ///
//  /// # Arguments
//  ///  * `screen` - Screen to draw on
//  ///  * `layout` - Layout to position and resize the component
//  ///  * `focus` - The currently focused component (should be ignored when this does not match the select id of self)
//  ///
//  /// # Returns
//  ///  * `None` - if no event was handles
//  ///  * `Some(Event)` - if an event was handled
//  fn draw<L: Layout>(&mut self, screen: &mut Screen<S>, layout: &mut L, focus: &mut SelectId) -> Option<Self::Event>;
//}

pub trait Component<S: Style>: Size {
  type Event: std::fmt::Debug;

  //fn handle_event(&mut self, e: &winit::event::Event<i32>) {
  //}
  //fn get_drawinfo(&self) -> Option<(MeshId, Scissor)> {
  //  None
  //}
  //fn get_selectinfo(&self) -> Option<(MeshId, Scissor)> {
  //  None
  //}

  //fn draw<L: Layout>(
  //  &mut self,
  //  screen: &mut Screen<S>,
  //  layout: &mut L,
  //  focus: &mut SelectId,
  //  e: Option<&winit::event::Event<i32>>,
  //) -> Option<Self::Event>;
  //{
  //match e {
  //  Some(e) => self.handle_event(e),
  //  None => {
  //    if let Some((mesh, scissor)) = self.get_drawinfo() {
  //      screen.push_draw(mesh, scissor);
  //    }
  //    if let Some((mesh, scissor)) = self.get_selectinfo() {
  //      screen.push_select(mesh, scissor);
  //    }
  //  }
  //}
  //}

  // TODO: stream::into with different result/event type
  fn enqueue<'a, R: std::fmt::Debug>(&mut self, s: Stream<'a, S, R>) -> Stream<'a, S, Self::Event> {
    s.with_result(None)
  }
}
