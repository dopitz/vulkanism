use crate::rect::Rect;

pub trait Size {
  /// Sets the size and position of a gui component
  fn rect(&mut self, rect: Rect) -> &mut Self;
  /// Gets the current size and position of the component
  fn get_rect(&self) -> Rect;

  /// Gets the ideal size of the component
  ///
  /// [Components](struct.Component.html) (and [Layouts](struct.Layout.html)) may define a size that is ideal for rendering it.
  /// In case of a Component this could mean that it then can be displayed without cutting anything of. 
  /// A Layout might track the size of Components and then be able to tell how large it has to be in order to fit all components adequately.
  ///
  /// A [Layout](struct.Layout.html) may use this size as a guide and will (implementation dependent try) to adhere by the component's ideal size.
  fn get_size_hint(&self) -> vkm::Vec2u;
}


