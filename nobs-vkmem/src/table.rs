use std::collections::{BTreeMap, HashMap};
use std::fmt::Write;
use std::rc::Weak;

use crate::bindtype::BindType;
use crate::memtype::Memtype;
use crate::BindInfoInner;
use crate::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord)]
struct Block {
  mem: vk::DeviceMemory,
  beg: vk::DeviceSize,
  end: vk::DeviceSize,
  pad: vk::DeviceSize,
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

#[derive(Debug, Clone)]
struct Node {
  next: Option<Block>,
  prev: Option<Block>,
}

#[derive(Debug, Clone)]
struct Occupied {
  handle: u64,
  next: Option<Block>,
  prev: Option<Block>,
}

struct Table {
  device: vk::Device,
  memtype: Memtype,

  pagesize: vk::DeviceSize,

  pages: HashMap<vk::DeviceMemory, HashMap<Block, Occupied>>,
  free: BTreeMap<Block, Node>,
}

impl Table {
  /// Allocates device memory
  ///
  /// Returns a block with the allocated size and the new device memory handle.
  /// Fails with [AllockError](../enum.Error.html) if the vulkan command failed.
  fn allocate_page(&self, pagesize: vk::DeviceSize) -> Result<Block, Error> {
    let alloc_info = vk::MemoryAllocateInfo {
      sType: vk::STRUCTURE_TYPE_MEMORY_ALLOCATE_INFO,
      pNext: std::ptr::null(),
      allocationSize: pagesize,
      memoryTypeIndex: self.memtype.index,
    };

    let mut handle = vk::NULL_HANDLE;

    vk_check!(vk::AllocateMemory(self.device, &alloc_info, std::ptr::null(), &mut handle)).map_err(|_| Error::AllocError)?;
    assert!(handle != vk::NULL_HANDLE);

    Ok(Block::new(handle, 0, pagesize, 0))
  }

  fn scan_bindinfos(infos: &[BindInfoInner], maxsize: Option<vk::DeviceSize>) -> (vk::DeviceSize, vk::DeviceSize, usize) {
    // use the largest alignment for all resources
    let alignment = infos
      .iter()
      .fold(0, |align, i| vk::DeviceSize::max(align, i.requirements.alignment));

    let mut count = 0;
    let mut size = 0;
    let maxsize = maxsize.unwrap_or(vk::DeviceSize::max_value());

    for i in infos.iter() {
      let pad = i.requirements.alignment - (size % i.requirements.alignment);
      if size + pad + i.size > maxsize {
        break;
      }
      size += pad + i.size;
      count += 1;
    }

    (alignment, size, count)
  }

  pub fn bind(&mut self, bindinfos: &[BindInfoInner]) -> Result<(), Error> {
    struct Group {
      b: usize,
      e: usize,
      block: (Option<Block>, Block, Option<Block>),
    };

    let mut groups = Vec::new();
    let mut g = Group {
      b: 0,
      e: bindinfos.len(),
      block: (None, Block::default(), None),
    };

    loop {
      let infos = &bindinfos[g.b..g.e];
      let (alignment, size, _) = Self::scan_bindinfos(infos, None);

      if let Some((b, n)) = self
        .free
        .range(Block::new(0, 0, size, 0)..)
        .take_while(|(b, n)| b.size() - (alignment - (b.beg % alignment)) >= size)
        .last()
        .map(|(b, n)| (*b, n.clone()))
      {
        self.free.remove(&b);
        groups.push(Group {
          b: g.b,
          e: g.e,
          block: (n.prev, b, n.next),
        });
        break;
      } else {
        if let Some((b, n)) = self.free.iter().next().map(|(b, n)| (*b, n.clone())) {
          let (a, s, c) = Self::scan_bindinfos(infos, Some(b.size()));
          if c != 0 {
            self.free.remove(&b);
            groups.push(Group {
              b: g.b,
              e: g.b + c,
              block: (n.prev, b, n.next),
            });

            g.b = g.b + c;
            continue;
          }
        }
        let b = self.allocate_page(vk::DeviceSize::max(size, self.pagesize))?;
        groups.push(Group {
          b: g.b,
          e: g.e,
          block: (None, b, None),
        });
        break;
      }
    }

    for g in groups.iter() {
      // bind
    }

    Ok(())
  }
}
