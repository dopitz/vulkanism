use std::collections::HashMap;

pub trait AssetType {
  type Types;
  fn load(id: &str) -> Self::Types;
}

pub struct At {}

impl AssetType for At {
  type Types = vk::Image;
  fn load(id: &str) -> Self::Types {
    println!("cat");
    0
  }
}

pub struct At2 {}

impl AssetType for At2 {
  type Types = usize;
  fn load(id: &str) -> Self::Types {
    println!("dog");
    0
  }
}

pub enum AtX {
  Image(<At as AssetType>::Types),
  Usize(<At2 as AssetType>::Types)
}

impl AssetType for AtX {
  type Types = Self;
  fn load(id: &str) -> Self::Types {
    AtX::Image(0)
  }
}

macro_rules! asset_type {
  (($name:ident, $ty:expr)*) => (
    
  )
}


pub struct Assets<T: AssetType> {
  assets: HashMap<String, T::Types>,
}

impl<T: AssetType> Assets<T> {
  pub fn contains(&self, id: &str) -> bool {
    self.assets.contains_key(id)
  }
  pub fn get(&mut self, id: &str) -> &T::Types {
    if self.contains(id) {
      self.assets.get(id).unwrap()
    } else {
      self.assets.insert(id.to_owned(), T::load(id));
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
