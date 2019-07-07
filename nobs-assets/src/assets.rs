use super::Update;
use std::collections::HashMap;

pub trait Asset: Sized {
  type Id: Clone + PartialEq + Eq + std::hash::Hash;

  fn load(id: &Self::Id, assets: &mut HashMap<Self::Id, Self>, up: &mut Update);
  fn free(id: &Self::Id, assets: &mut HashMap<Self::Id, Self>, up: &mut Update);

  fn get<'a>(id: &Self::Id, assets: &'a mut HashMap<Self::Id, Self>, up: &mut Update) -> &'a Self {
    if !assets.contains_key(id) {
      Self::load(id, assets, up);
    }

    assets.get(id).unwrap()
  }
}
