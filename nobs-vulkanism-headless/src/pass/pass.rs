use crate::cmd::stream::*;
use crate::cmd::CmdBuffer;
use vk;

pub trait PassId: std::hash::Hash + PartialEq + Eq + Clone + Copy {
  type Mask: PassMask<Self>;
}

pub trait PassMask<T: PassId>: std::hash::Hash + Default + PartialEq + Eq + Clone + Copy {
  fn contains(&self, id: T) -> bool;
  fn is_empty(&self) -> bool;
  fn add(&mut self, id: T);
  fn remove(&mut self, id: T);
}

pub trait Pass: StreamPush {
  fn resize(&mut self, size: vk::Extent2D);
}
