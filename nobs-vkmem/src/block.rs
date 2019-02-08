use vk;
use vk::DeviceSize;

/// Block of device memory used for tracking space on allocated device memory
///
/// Stores the byte offsets `begin` and `end` into device memory with handle `mem`
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Ord, Hash)]
pub struct Block {
  pub begin: DeviceSize,
  pub end: DeviceSize,
  pub mem: vk::DeviceMemory,
}

impl PartialOrd for Block {
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    use std::cmp::Ordering::*;
    match self.size().cmp(&other.size()) {
      Equal => Some(other.begin.cmp(&self.begin)),
      Greater => Some(Greater),
      Less => Some(Less),
    }
  }
}

impl Block {
  pub fn new(begin: DeviceSize, end: DeviceSize, mem: vk::DeviceMemory) -> Block {
    Block { begin, end, mem }
  }
  pub fn with_size(offset: DeviceSize, size: DeviceSize, mem: vk::DeviceMemory) -> Block {
    Self::new(offset, offset + size, mem)
  }

  /// Size of the block in bytes
  pub fn size(&self) -> DeviceSize {
    self.end - self.begin
  }
  /// Size of the block in bytes after padding the begin to the requested alignment
  pub fn size_aligned(&self, alignment: DeviceSize) -> DeviceSize {
    self.size().wrapping_sub(self.begin % alignment)
  }

  /// Checks if two Blocks are overlapping
  ///
  /// Blocks overlap if, they share a common region on the SAME device memory allocation
  pub fn overlapping(a: &Block, b: &Block) -> bool {
    a.mem == b.mem && a.begin <= b.end && b.begin <= a.end
  }

  /// Merges the two blocks together, if they are overlapping
  pub fn merge(a: &Block, b: &Block) -> Option<Block> {
    match Self::overlapping(a, b) {
      true => Some(Block {
        begin: DeviceSize::min(a.begin, b.begin),
        end: DeviceSize::max(a.end, b.end),
        mem: a.mem,
      }),
      false => None,
    }
  }
}
