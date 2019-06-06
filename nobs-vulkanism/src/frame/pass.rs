use vk;

pub trait PassId: std::hash::Hash + PartialEq + Eq + Clone + Copy {
  type Mask: PassMask<Self>;
}

pub trait PassMask<T: PassId>: std::hash::Hash + PartialEq + Eq + Clone + Copy {
  fn contains(&self, id: T) -> bool;
  fn add(self, id: T) -> Self;
  fn remove(self, id: T) -> Self;
}

#[repr(u8)]
#[derive(Hash, PartialEq, Eq, Clone, Copy)]
pub enum MyPassId {
  Present = 1,
  Select = 2,
  Shadow = 4,
}

impl PassId for MyPassId {
  type Mask = MyPassMask;
}

#[derive(Hash, PartialEq, Eq, Clone, Copy)]
pub struct MyPassMask {
  mask: u8,
}

impl PassMask<MyPassId> for MyPassMask {
  fn contains(&self, id: MyPassId) -> bool {
    (self.mask & id as u8) != 0
  }
  fn add(self, id: MyPassId) -> Self {
    Self {
      mask: self.mask & id as u8,
    }
  }
  fn remove(self, id: MyPassId) -> Self {
    Self {
      mask: self.mask & !(id as u8),
    }
  }
}

pub trait Pass {
  fn run(&mut self, cmds: vk::cmd::Pool, batch: &mut vk::cmd::Frame);

  fn resize(mut self, size: vk::Extent2D) -> Self;
}
