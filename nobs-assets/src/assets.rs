use super::Update;
use std::collections::HashMap;

pub trait AssetType {
  type Type;
  fn load(id: &str, up: &mut Update) -> Self::Type;
  fn free(asset: Self::Type, up: &mut Update);
}

macro_rules! asset_type {
  ($name:ident, $(($atname:ident, $attype:ty)),+) => (
    pub enum $name {
      $(
        $atname(<$attype as AssetType>::Type),
      )+
    }
  )
}

pub struct Assets<T: AssetType> {
  device: vk::Device,
  mem: vk::mem::Mem,

  assets: HashMap<String, T::Type>,
  pub up: Update,
}

impl<T: AssetType> Assets<T> {
  pub fn new(device: vk::Device, mem: vk::mem::Mem) -> Self {
    Assets {
      device,
      mem: mem.clone(),
      assets: Default::default(),
      up: Update::new(mem),
    }
  }

  pub fn contains(&self, id: &str) -> bool {
    self.assets.contains_key(id)
  }
  pub fn get(&mut self, id: &str) -> &T::Type {
    if self.contains(id) {
      self.assets.get(id).unwrap()
    } else {
      self.assets.insert(id.to_owned(), T::load(id, &mut self.up));
      self.assets.get(id).unwrap()
    }
  }
}
