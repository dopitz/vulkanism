use super::Update;
use std::collections::HashMap;

pub trait AssetPool<T: Asset> {
  fn get(&self, id: &T::Id) -> Option<&T>;
  fn insert(&mut self, id: T::Id, asset: T);
  fn remove(&mut self, id: &T::Id) -> Option<T>;

  fn load(&mut self, id: &T::Id, up: &mut Update) -> &T {
    if self.get(id).is_none() {
      self.insert(id.clone(), T::load(id, up));
    }

    self.get(id).unwrap()
  }
  fn free(&mut self, id: &T::Id, up: &mut Update) {
    if let Some(a) = self.remove(id) {
      a.free(up);
    }
  }
}

impl<T: Asset> AssetPool<T> for HashMap<T::Id, T> {
  fn get(&self, id: &T::Id) -> Option<&T> {
    self.get(id)
  }
  fn insert(&mut self, id: T::Id, asset: T) {
    self.insert(id, asset);
  }
  fn remove(&mut self, id: &T::Id) -> Option<T> {
    self.remove(id)
  }
}

pub trait Asset : Sized {
  type Id: Clone + PartialEq + Eq + std::hash::Hash;

  fn load(id: &Self::Id, up: &mut Update) -> Self;
  fn free(self, up: &mut Update);
}
