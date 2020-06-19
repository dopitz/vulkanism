use crate::component::Component;
use crate::component::Size;
use crate::style::Style;
use crate::Rect;
use vk::cmd::commands::Scissor;

#[derive(Debug, Copy, Clone)]
pub struct AaBb {
  lo: vkm::Vec2i,
  hi: vkm::Vec2i,
}

impl Default for AaBb {
  fn default() -> Self {
    Self {
      lo: vec2!(i32::max_value()),
      hi: vec2!(i32::min_value()),
    }
  }
}

impl AaBb {
  pub fn union(&mut self, other: &Self) -> &mut Self {
    self.lo = vkm::Vec2::min(self.lo, other.lo);
    self.hi = vkm::Vec2::max(self.hi, other.hi);
    self
  }

  pub fn size(&self) -> vkm::Vec2u {
    vkm::Vec2::clamp(self.hi.into() - self.lo.into(), vec2!(0), vec2!(i64::max_value())).into()
  }
}

impl From<Rect> for AaBb {
  fn from(r: Rect) -> Self {
    Self {
      lo: r.position,
      hi: r.position + r.size.into(),
    }
  }
}

/// Defines rules for positioning and resizing gui components, when they are adden to a [Window](struct.Window.html).
///
/// Layouting gui components has to be incremental.
/// This means that the layout for a single components needs to be decided without knolwledge about other components.
/// The layouts internal state must be used instead.
pub trait Layout: Size {
  /// Restarts the layout
  fn restart(&mut self);

  /// Applys layout to componet
  ///
  /// This function should modify the components layout (position and size) using [Component::rect](trait.Component.html#method.rect).
  ///
  /// # Returns
  /// The scissor rect for the component
  fn layout(&mut self, c: &mut dyn Size) -> Scissor;

  /// Get the overlapping scissor rect from the layouts draw area  and `rect`
  fn get_scissor(&self, mut rect: Rect) -> Scissor {
    let lo = vkm::Vec2::clamp(self.get_rect().position, vec2!(0), vec2!(i32::max_value()));
    let hi = lo + self.get_rect().size.into();
    rect.position = vkm::Vec2::clamp(rect.position, lo, hi);
    rect.size = (vkm::Vec2::clamp(rect.position + rect.size.into(), lo, hi) - rect.position).into();
    Scissor::with_rect(rect.into())
  }
}

/// Float layout that does not modify componets
#[derive(Debug, Default, Clone, Copy)]
pub struct FloatLayout {
  rect: Rect,
  current: AaBb,
}

impl Size for FloatLayout {
  fn set_rect(&mut self, rect: Rect) {
    self.rect = rect;
  }

  fn get_rect(&self) -> Rect {
    self.rect
  }

  fn get_size_hint(&self) -> vkm::Vec2u {
    self.current.size()
  }
}

impl Layout for FloatLayout {
  fn restart(&mut self) {
    self.current = Default::default();
  }

  fn layout(&mut self, c: &mut dyn Size) -> Scissor {
    let r = c.get_rect();
    c.set_rect(r);
    self.current.union(&r.into());
    self.get_scissor(r)
  }
}

impl From<Rect> for FloatLayout {
  fn from(rect: Rect) -> Self {
    Self {
      rect,
      current: Default::default(),
    }
  }
}

/// Column layout that arranges components in a single column from top first to bottom last
#[derive(Debug, Default, Clone, Copy)]
pub struct ColumnLayout {
  rect: Rect,
  top: u32,

  current: AaBb,
}

impl Size for ColumnLayout {
  fn set_rect(&mut self, rect: Rect) {
    self.rect = rect;
  }

  fn get_rect(&self) -> Rect {
    self.rect
  }

  fn get_size_hint(&self) -> vkm::Vec2u {
    self.current.size()
  }
}

impl Layout for ColumnLayout {
  fn restart(&mut self) {
    self.top = 0;
    self.current = Default::default();
  }

  fn layout(&mut self, c: &mut dyn Size) -> Scissor {
    let mut rect = Rect::new(self.rect.position + vec2!(0, self.top as i32), c.get_size_hint());
    if rect.size.x == 0 {
      rect.size.x = self.rect.size.x;
    }
    self.top += rect.size.y;
    self.current.union(&rect.into());

    if rect.size.x >= self.rect.size.x {
      rect.size.x = self.rect.size.x;
    }
    c.set_rect(rect);
    self.get_scissor(rect)
  }
}

impl From<Rect> for ColumnLayout {
  fn from(rect: Rect) -> Self {
    Self {
      rect,
      top: 0,
      current: Default::default(),
    }
  }
}
