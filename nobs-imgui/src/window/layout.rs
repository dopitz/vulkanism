use super::Component;
use crate::rect::Rect;
use crate::style::Style;
use vk::cmd::commands::Scissor;

/// Defines rules for positioning and resizing gui components, when they are adden to a [Window](struct.Window.html).
///
/// Layouting gui components has to be incremental.
/// This means that the layout for a single components needs to be decided without knolwledge about other components.
/// The layouts internal state must be used instead.
pub trait Layout: Default {
  fn new() -> Self {
    Default::default()
  }
  /// Resets the layout
  ///
  /// # Arguments
  /// * `rect` - The draw area which may be used by components
  fn reset(&mut self, rect: Rect);
  /// Gets the draw area
  fn get_rect(&self) -> Rect;
  /// Applys layout to componet
  ///
  /// This function should modify the components layout (position and size) using [Component::rect](trait.Component.html#method.rect).
  ///
  /// # Returns
  /// The scissor rect for the component
  fn apply<S: Style, C: Component<S>>(&mut self, c: &mut C) -> Scissor;
}

/// Float layout that does not modify componets
#[derive(Default)]
pub struct FloatLayout {
  rect: Rect,
}

impl Layout for FloatLayout {
  fn reset(&mut self, rect: Rect) {
    self.rect = rect;
  }

  fn get_rect(&self) -> Rect {
    self.rect
  }

  fn apply<S: Style, C: Component<S>>(&mut self, _c: &mut C) -> Scissor {
    Scissor::with_rect(self.rect.into())
  }
}

/// Column layout that arranges components in a single column from top first to bottom last
#[derive(Default)]
pub struct ColumnLayout {
  rect: Rect,
  top: u32,
}

impl Layout for ColumnLayout {
  fn reset(&mut self, rect: Rect) {
    self.rect = rect;
  }

  fn get_rect(&self) -> Rect {
    self.rect
  }

  fn apply<S: Style, C: Component<S>>(&mut self, c: &mut C) -> Scissor {
    let mut rect = Rect::new(self.rect.position + vec2!(0, self.top as i32), c.get_size_hint());
    if rect.size.x == 0 {
      rect.size.x = self.rect.size.x;
    }
    if rect.size.x >= self.rect.size.x {
      rect.size.x = self.rect.size.x;
    }
    if self.top + rect.size.y >= self.rect.size.y {
      rect.size.y = self.rect.size.y - self.top;
    }
    c.rect(rect);
    self.top += rect.size.y;

    Scissor::with_rect(rect.into())
  }
}
