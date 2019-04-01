use std::collections::{BTreeSet, HashMap};
use std::fmt::Write;

use crate::bindtype::BindType;
use crate::memtype::Memtype;
use crate::BindInfoInner;
use crate::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, Hash)]
struct Block {
  mem: vk::DeviceMemory,
  idx: usize,
  size: vk::DeviceSize,
}

impl PartialOrd for Block {
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    self.size.partial_cmp(&other.size)
  }
}

impl Default for Block {
  fn default() -> Block {
    Block { mem: 0, idx: 0, size: 0 }
  }
}

impl Block {
  pub fn new(mem: vk::DeviceMemory, idx: usize, size: vk::DeviceSize) -> Block {
    Block { mem, idx, size }
  }

  pub fn from_size(size: vk::DeviceSize) -> Block {
    Self::new(0, 0, size)
  }
}

struct Page {
  blocks: Vec<u8>,
}

struct Table {
  device: vk::Device,
  memtype: Memtype,

  pagesize: vk::DeviceSize,
  blocksize: vk::DeviceSize,

  pages: HashMap<vk::DeviceMemory, Page>,
  free: BTreeSet<Block>,
  alloc: HashMap<u64, Block>,
}

impl Table {
  pub fn bind(&mut self, bindinfos: &[BindInfoInner], bindtype: BindType) -> Result<(), Error> {
    let mut blocks = Vec::with_capacity(bindinfos.len());
    blocks.resize(bindinfos.len(), Block::default());

    /// Continuous bindinfos that can be bound to the same free block
    struct Group {
      b: usize,
      e: usize,
      block: Block,
    }

    let mut groups = Vec::new();
    let mut g = Group {
      b: 0,
      e: bindinfos.len(),
      block: Block::default(),
    };

    loop {
      // find binding positions for every block
      let bindinfos = &bindinfos[g.b..g.e];
      let blocks = &mut blocks[g.b..g.e];

      let (alignment, size, _) = Self::compute_blocks(bindinfos, blocks, None);

      if let Some(b) = self
        .free
        .range(Block::from_size(size)..)
        .take_while(|b| b.size.wrapping_sub(b.idx as u64 * self.blocksize) % alignment >= size)
        .next()
        .cloned()
      {
        // we found a good block, so we use it
        self.free.remove(&b);
        groups.push(Group { b: g.b, e: g.e, block: b });
        break;
      } else {

      }
    }

    Ok(())
  }

  /// Compute blocks from [BindInfos](struct.BindInfo.html)
  ///
  /// Requires that `bindinfos.len() == blocks.len()`.
  ///
  /// Sets the begin and end field in every block. The offsets are computed so that every block is aligned correctly w/r to their BindInfo's reqirements.
  /// The block's begin and end are computed to base address 0, e.g. the first block starts with `begin == 0`.
  /// Returns the reqired alignment of the first block and the required total size of all blocks.
  fn scan_bindinfos(
    bindinfos: &[BindInfoInner],
    blocks: &mut [(vk::DeviceSize, vk::DeviceSize)],
    size: Option<vk::DeviceSize>,
  ) -> (vk::DeviceSize, usize) {
    // use the largest alignment for all resources
    let alignment = bindinfos
      .iter()
      .fold(0, |align, i| vk::DeviceSize::max(align, i.requirements.alignment));

    let mut count = 0;

    for (block, info) in blocks.iter_mut().zip(bindinfos.iter()) {





      let begin = match prev.end % info.requirements.alignment {
        0 => prev.end,
        modulo => prev.end + info.requirements.alignment - modulo,
      };

      *block = Block::with_size(begin, info.size, 0);
      prev = *block;
      count += 1;

      if let Some(s) = size {
        if prev.end >= s {
          break;
        }
      }
    }

    (alignment, prev.end, count)
  }

  //  pub fn bindx(&mut self, bindinfos: &[BindInfo], bindtype: BindType) -> Result<(), Error> {
  //    let mut blocks = Vec::with_capacity(bindinfos.len());
  //    blocks.resize(bindinfos.len(), Block::new(0, 0, 0));
  //
  //    /// Continuous bindinfos that can be bound to the same free block
  //    struct Group {
  //      b: usize,
  //      e: usize,
  //      free: Block,
  //    }
  //
  //    let mut groups = Vec::new();
  //    let mut g = Group {
  //      b: 0,
  //      e: bindinfos.len(),
  //      free: Block::new(0, 0, 0),
  //    };
  //
  //    // find binding positions for every block
  //    match bindtype {
  //      BindType::Scatter | BindType::Block => loop {
  //        let bindinfos = &bindinfos[g.b..g.e];
  //        let blocks = &mut blocks[g.b..g.e];
  //
  //        let (alignment, size, _) = Self::compute_blocks(bindinfos, blocks, None);
  //
  //        if let Some(free) = self
  //          .free
  //          .range(Block::with_size(0, size + alignment, 0)..)
  //          .take_while(|b| b.size_aligned(alignment) >= size)
  //          .last()
  //          .cloned()
  //        {
  //          // we found a good block, so we use it
  //          self.free.remove(&free);
  //          groups.push(Group {
  //            b: g.b,
  //            e: g.e,
  //            free: free,
  //          });
  //          break;
  //        } else {
  //          // no good match
  //          // find the largest free block and fill it with as many bindings as possible
  //
  //          let free = match bindtype {
  //            BindType::Block => self.allocate_page(std::cmp::max(size, self.pagesize))?,
  //            BindType::Scatter => match self.free.iter().next().cloned().and_then(|b| self.free.take(&b)) {
  //              Some(largest) => {
  //                let (alignment, size, count) = Self::compute_blocks(bindinfos, blocks, Some(largest.size_aligned(alignment)));
  //
  //                match count {
  //                  0 => self.allocate_page(std::cmp::max(size, self.pagesize))?,
  //                  _ => largest,
  //                }
  //              }
  //              None => self.allocate_page(std::cmp::max(size, self.pagesize))?,
  //            },
  //            _ => panic!("unreachable"),
  //          };
  //
  //          let (alignment, size, count) = Self::compute_blocks(bindinfos, blocks, Some(free.size_aligned(alignment)));
  //
  //          groups.push(Group {
  //            b: g.b,
  //            e: g.b + count,
  //            free: free,
  //          });
  //
  //          if count == bindinfos.len() {
  //            break;
  //          }
  //        }
  //      },
  //      BindType::Minipage => {
  //        let (alignment, size, _) = Self::compute_blocks(bindinfos, blocks, None);
  //        let page = self.allocate_page(size)?;
  //        groups.push(Group {
  //          b: g.b,
  //          e: g.e,
  //          free: free,
  //        });
  //      }
  //    }
  //
  //    // bind blocks to memory, create paddings and remainder free blocks
  //    for g in groups.iter() {
  //      let blocks = &mut blocks[g.b..g.e];
  //
  //      let offset = dst.begin + (alignment + dst.size_aligned(alignment) - dst.size()) % alignment;
  //      let mem = dst.mem;
  //
  //      for b in blocks.iter_mut() {
  //        b.begin += offset;
  //        b.end += offset;
  //        b.mem = mem;
  //      }
  //
  //      assert!(blocks.last().unwrap().end <= dst.end);
  //      Block::new(blocks.last().unwrap().end, dst.end, dst.mem)
  //    }
  //
  //    Ok(())
  //  }
}
