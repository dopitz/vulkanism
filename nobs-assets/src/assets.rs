use super::Update;
use std::collections::HashMap;

pub trait AssetType {
  type Types;
  fn load(id: &str, up: &mut Update) -> Self::Types;
}

macro_rules! asset_type {
  ($name:ident, $(($atname:ident, $attype:ty)),+) => (
    pub enum $name {
      $(
        $atname(<$attype as AssetType>::Types),
      )+
    }
  )
}

pub struct Assets<T: AssetType> {
  device: vk::Device,
  mem: vk::mem::Mem,

  assets: HashMap<String, T::Types>,
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
  pub fn get(&mut self, id: &str) -> &T::Types {
    if self.contains(id) {
      self.assets.get(id).unwrap()
    } else {
      self.assets.insert(id.to_owned(), T::load(id, &mut self.up));
      self.assets.get(id).unwrap()
    }
  }
}

#[cfg(test)]
mod tests {
  #[test]
  fn aaa() {
    assert_eq!(3, 3);
  }
}
