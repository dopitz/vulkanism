use super::Component;
use crate::rect::Rect;
use crate::style::Style;
use vk::cmd::commands::Scissor;

/// Defines rules for positioning and resizing gui components, when they are adden to a [Window](struct.Window.html).
///
/// Layouting gui components has to be incremental.
/// This means that the layout for a single components needs to be decided without knolwledge about other components.
/// The layouts internal state must be used instead.
pub trait Layout {
  /// Restarts the layout
  fn restart(&mut self);

  /// Sets the draw area
  fn set_rect(&mut self, rect: Rect);
  /// Gets the draw area
  fn get_rect(&self) -> Rect;

  /// Applys layout to componet
  ///
  /// This function should modify the components layout (position and size) using [Component::rect](trait.Component.html#method.rect).
  ///
  /// # Returns
  /// The scissor rect for the component
  fn apply<S: Style, C: Component<S>>(&mut self, c: &mut C) -> Scissor;

  /// Get the overlapping scissor rect from the layouts draw area  and `rect`
  fn get_scissor(&self, mut rect: Rect) -> Scissor {
    let lo = vkm::Vec2::clamp(self.get_rect().position, vec2!(0), vec2!(i32::max_value()));
    let hi = lo + self.get_rect().size.into();
    rect.position = vkm::Vec2::clamp(rect.position, lo, hi);
    rect.size = (vkm::Vec2::clamp(rect.position + rect.size.into(), lo, hi) - rect.position).into();
    Scissor::with_rect(rect.into())
  }

  /// Get the ideal layout size
  ///
  /// The result of this function may only be defined AFTER all components of this have been draw.
  fn get_size_hint(&self) -> vkm::Vec2u;
}

/// Float layout that does not modify componets
#[derive(Default)]
pub struct FloatLayout {
  rect: Rect,
  lo: vkm::Vec2i,
  hi: vkm::Vec2i,
}

impl Layout for FloatLayout {
  fn restart(&mut self) {
    self.lo = vec2!(i32::min_value());
    self.hi = vec2!(i32::max_value());
  }

  fn set_rect(&mut self, rect: Rect) {
    self.rect = rect;
  }

  fn get_rect(&self) -> Rect {
    self.rect
  }

  fn apply<S: Style, C: Component<S>>(&mut self, c: &mut C) -> Scissor {
    let r = c.get_rect();
    self.lo = vkm::Vec2::max(self.lo, r.position);
    self.hi = vkm::Vec2::min(self.hi, r.position + r.size.into());
    self.get_scissor(r)
  }

  fn get_size_hint(&self) -> vkm::Vec2u {
    (self.hi - self.lo).into()
  }
}

impl From<Rect> for FloatLayout {
  fn from(rect: Rect) -> Self {
    Self {
      rect,
      lo: vec2!(i32::min_value()),
      hi: vec2!(i32::max_value()),
    }
  }
}

/// Column layout that arranges components in a single column from top first to bottom last
#[derive(Default)]
pub struct ColumnLayout {
  rect: Rect,
  top: u32,

  lo: vkm::Vec2i,
  hi: vkm::Vec2i,
}

impl Layout for ColumnLayout {
  fn restart(&mut self) {
    self.top = 0;
    self.lo = vec2!(i32::min_value());
    self.hi = vec2!(i32::max_value());
  }

  fn set_rect(&mut self, rect: Rect) {
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
    c.rect(rect);
    self.top += rect.size.y;
    self.lo = vkm::Vec2::max(self.lo, rect.position);
    self.hi = vkm::Vec2::min(self.hi, rect.position + rect.size.into());

    self.get_scissor(rect)
  }

  fn get_size_hint(&self) -> vkm::Vec2u {
    (self.hi - self.lo).into()
  }
}

impl From<Rect> for ColumnLayout {
  fn from(rect: Rect) -> Self {
    Self {
      rect,
      top: 0,
      lo: vec2!(i32::min_value()),
      hi: vec2!(i32::max_value()),
    }
  }
}
