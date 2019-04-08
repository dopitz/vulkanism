#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, Hash)]
pub struct Block {
  pub mem: vk::DeviceMemory,
  pub beg: vk::DeviceSize,
  pub end: vk::DeviceSize,
  pub pad: vk::DeviceSize,
}

impl Default for Block {
  fn default() -> Block {
    Block {
      mem: 0,
      beg: 0,
      end: 0,
      pad: 0,
    }
  }
}

impl Block {
  pub fn new(mem: vk::DeviceMemory, beg: vk::DeviceSize, end: vk::DeviceSize, pad: vk::DeviceSize) -> Block {
    Block { mem, beg, end, pad }
  }

  pub fn size(&self) -> vk::DeviceSize {
    self.end - self.beg
  }

  pub fn size_padded(&self) -> vk::DeviceSize {
    self.end - self.beg - self.pad
  }
}

impl PartialOrd for Block {
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    use std::cmp::Ordering::*;
    match self.size().cmp(&other.size()) {
      Equal => Some(other.beg.cmp(&self.beg)),
      Greater => Some(Greater),
      Less => Some(Less),
    }
  }
}

#[derive(Debug, Clone, Copy)]
pub enum BlockType {
  Free(Block),
  Occupied(Block),
}

impl BlockType {
  pub fn get(&self) -> Block {
    match self {
      BlockType::Free(b) => *b,
      BlockType::Occupied(b) => *b,
    }
  }
}

impl std::fmt::Display for BlockType {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    let (t, b) = match self {
      BlockType::Free(b) => ("Free    ", b),
      BlockType::Occupied(b) => ("Occupied", b),
    };
    write!(f, "{}: {} - {} ({})", t, b.beg, b.end, b.size())
  }
}

