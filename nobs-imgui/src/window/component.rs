use crate::rect::Rect;

pub trait Component {
  fn rect(&mut self, rect: Rect) -> &mut Self;
  fn get_rect(&self) -> Rect;

  fn get_size_hint(&self) -> vkm::Vec2u;

  fn get_mesh(&self) -> usize;
}
