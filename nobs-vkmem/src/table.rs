use std::collections::{BTreeMap, HashMap};
use std::fmt::Write;

use crate::bindtype::BindType;
use crate::memtype::Memtype;
use crate::BindInfoInner;
use crate::Error;
use crate::Handle;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, Hash)]
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

#[derive(Debug, Clone, Copy)]
enum BlockType {
  Free(Block),
  Occupied(Block),
}

#[derive(Debug, Clone)]
struct Node {
  prev: Option<BlockType>,
  next: Option<BlockType>,
}

#[derive(Debug, Clone)]
struct Binding {
  handle: u64,
  node: Node,
}

struct Table {
  device: vk::Device,
  memtype: Memtype,

  pagesize: vk::DeviceSize,

  pages: HashMap<vk::DeviceMemory, HashMap<Block, Binding>>,
  free: BTreeMap<Block, Node>,
}

impl Table {
  fn get_padding(offset: vk::DeviceSize, align: vk::DeviceSize) -> vk::DeviceSize {
    match offset % align {
      0 => 0,
      a => align - a,
    }
  }

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
      let pad = Self::get_padding(size, i.requirements.alignment);
      if size + pad + i.size > maxsize {
        break;
      }
      size += pad + i.size;
      count += 1;
    }

    (alignment, size, count)
  }

  pub fn bind(&mut self, bindinfos: &[BindInfoInner], bindtype: BindType) -> Result<(), Error> {
    struct Group {
      b: usize,
      e: usize,
      block: Block,
      node: Node,
    };

    let mut groups = Vec::new();
    let mut g = Group {
      b: 0,
      e: bindinfos.len(),
      block: Block::default(),
      node: Node { prev: None, next: None },
    };

    // find groups of bindinfos, that we can bind together to a free block of memory
    loop {
      let infos = &bindinfos[g.b..g.e];
      let (alignment, size, _) = Self::scan_bindinfos(infos, None);

      if let Some((b, n)) = self
        .free
        .range(Block::new(0, 0, size, 0)..)
        .take_while(|(b, n)| groups.iter().any(|g: &Group| **b == g.block) || b.size() - Self::get_padding(b.beg, alignment) >= size)
        .last()
        .map(|(b, n)| (*b, n.clone()))
      {
        groups.push(Group {
          b: g.b,
          e: g.e,
          block: b,
          node: n,
        });
        break;
      } else {
        // Only if we are allowed to split,
        // find the biggest block and bind as many handles as possible there.
        if let BindType::Scatter {} = bindtype {
          if let Some((b, n)) = self.free.iter().next().map(|(b, n)| (*b, n.clone())) {
            let (a, s, c) = Self::scan_bindinfos(infos, Some(b.size()));
            // Only if we find a free block that is large enough, we continue the loop
            // If not, we allocate a new page and put everything there
            if c != 0 {
              self.free.remove(&b);
              groups.push(Group {
                b: g.b,
                e: g.b + c,
                block: b,
                node: n,
              });

              g.b = g.b + c;
              continue;
            }
          }
        }

        // Allocate a new page
        // if it fails, return the free block of every group to the table
        let b = self.allocate_page(vk::DeviceSize::max(size, self.pagesize))?;
        groups.push(Group {
          b: g.b,
          e: g.e,
          block: b,
          node: Node { prev: None, next: None },
        });
        break;
      }
    }

    let mut blocks = Vec::with_capacity(bindinfos.len());
    let mut err = false;

    // bind the resources
    for g in groups.iter() {
      let infos = &bindinfos[g.b..g.e];

			debug_assert!(g.node.prev)
      let mut offset = match &g.node.prev {
        Some(prev) => match prev {
          BlockType::Free(b) => b.end,
          BlockType::Occupied(b) => b.end,
        },
        None => g.block.beg,
      };

      let mem = g.block.mem;

      for i in infos.iter() {
        let pad = Self::get_padding(offset, i.requirements.alignment);

        blocks.push(Block::new(mem, offset, offset + pad + i.size, pad));

        match i.handle {
          Handle::Buffer(h) => vk_check!(vk::BindBufferMemory(self.device, h, mem, offset + pad)),
          Handle::Image(h) => vk_check!(vk::BindImageMemory(self.device, h, mem, offset + pad)),
        }
        .map_err(|_| Error::BindMemoryFailed)?;

        offset = offset + pad + i.size;
      }
    }

    // update free/occupied blocks
    for g in groups.iter() {
      let infos = &bindinfos[g.b..g.e];
      let blocks = &blocks[g.b..g.e];

      if let Some(n) = self.free.remove(&g.block) {
        match n.prev {
          Some(prev) => self.get_node(prev).next = Some(BlockType::Occupied(*blocks.first().unwrap())),
          None => (),
        }

          self.insert(
            BlockType::Occupied(*blocks.first().unwrap()),
            Node {
              prev: BlockType::Occupied(blocks[i - 1]),
              next: BlockType::Occupied(blocks[i + 1]),
            },
          )


        for i in 1..blocks.len() - 1 {
          self.insert(
            BlockType::Occupied(blocks[i]),
            Node {
              prev: BlockType::Occupied(blocks[i - 1]),
              next: BlockType::Occupied(blocks[i + 1]),
            },
          )
        }

        match n.next {
          Some(next) => self.get_node(next).prev = Some(BlockType::Occupied(*blocks.last().unwrap())),
          None => (),
        }
      }
    }

    Ok(())
  }

  fn get_node(&mut self, b: BlockType) -> Option<&mut Node> {
    match b {
      BlockType::Free(b) => return self.free.get_mut(&b),
      BlockType::Occupied(b) => self
        .pages
        .get_mut(&b.mem)
        .and_then(|p| p.get_mut(&b))
        .and_then(|x| Some(&mut x.node)),
    }
  }

  fn insert(&mut self, b: BlockType, n: Node, h: Option<u64>) {
    if let Some(prev) = n.prev {
      if let Some(n) = self.get_node(prev) {
        n.next = Some(b);
      }
    }
    if let Some(next) = n.next {
      if let Some(n) = self.get_node(next) {
        n.prev = Some(b);
      }
    }

    match b {
      BlockType::Free(b) => {
        self.free.insert(b, n);
      }
      BlockType::Occupied(b) => {
        self.pages.get_mut(&b.mem).and_then(|p| {
          p.insert(
            b,
            Binding {
              handle: h.unwrap(),
              node: n,
            },
          )
        });
      }
    };
  }

  fn set_nodeptr(&mut self, node: &Node, new: Option<BlockType>) {
    match node.prev {
      Some(BlockType::Free(b)) => {
        self.free.entry(b).and_modify(|n| n.next = new);
      }
      Some(BlockType::Occupied(b)) => {
        self.pages.entry(b.mem).and_modify(|p| {
          p.entry(b).and_modify(|b| b.node.next = new);
        });
      }
      _ => (),
    }

    match node.next {
      Some(BlockType::Free(b)) => {
        self.free.entry(b).and_modify(|n| n.prev = new);
      }
      Some(BlockType::Occupied(b)) => {
        self.pages.entry(b.mem).and_modify(|p| {
          p.entry(b).and_modify(|b| b.node.prev = new);
        });
      }
      _ => (),
    }
  }

  fn remove_block(&mut self, b: &BlockType) {
    match b {
      BlockType::Free(b) => {
        if let Some(n) = self.free.remove(b) {
          self.set_nodeptr(&n, None);
        }
      }
      BlockType::Occupied(b) => {
        if let Some(n) = self.pages.get_mut(&b.mem).and_then(|p| p.remove(b)) {
          self.set_nodeptr(&n.node, None);
        }
      }
    }
  }

  fn replace_block(&mut self, old: &BlockType, new: &BlockType) {
    match old {
      BlockType::Free(b) => {
        if let Some(n) = self.free.remove(b) {
          self.set_nodeptr(&n, Some(*new));
          self.free.insert(*b, n);
        }
      }
      BlockType::Occupied(b) => {
        if let Some(n) = self.pages.get_mut(&b.mem).and_then(|p| p.remove(b)) {
          self.set_nodeptr(&n.node, Some(*new));
          self.pages.get_mut(&b.mem).and_then(|p| p.insert(*b, n));
        }
      }
    }
  }
}
