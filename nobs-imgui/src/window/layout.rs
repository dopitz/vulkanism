use super::Component;
use crate::rect::Rect;
use vk::cmd::commands::Scissor;

pub trait Layout: Default {
  fn new() -> Self {
    Default::default()
  }
  fn reset(&mut self, rect: Rect);
  fn get_rect(&self) -> Rect;
  fn push<T: Component>(&mut self, c: &mut T) -> (Scissor, usize);
}

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

  fn push<T: Component>(&mut self, c: &mut T) -> (Scissor, usize) {
    (Scissor::with_rect(c.get_rect().into()), c.get_mesh())
  }
}

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

  fn push<T: Component>(&mut self, c: &mut T) -> (Scissor, usize) {
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

    (Scissor::with_rect(rect.into()), c.get_mesh())
  }
}
