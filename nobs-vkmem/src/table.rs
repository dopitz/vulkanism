use std::collections::{BTreeMap, HashMap};
use std::fmt::Write;

use crate::bindinfo::BindInfoInner;
use crate::bindtype::BindType;
use crate::block::Block;
use crate::block::BlockType;
use crate::memtype::Memtype;
use crate::Error;
use crate::Handle;

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

pub struct Table {
  device: vk::Device,
  memtype: Memtype,

  pagesize: vk::DeviceSize,

  pages: HashMap<vk::DeviceMemory, HashMap<Block, Binding>>,
  bindings: HashMap<u64, Block>,
  free: BTreeMap<Block, Node>,
}

impl Drop for Table {
  fn drop(&mut self) {
    for (mem, _) in self.pages.iter() {
      vk::FreeMemory(self.device, *mem, std::ptr::null());
    }
  }
}

impl Table {
  /// Creates a new page table with the desired page size.
  ///
  /// We do not need to check for the minimum page size, since [Allocator](../struct.Allocator.html) already does that, and we don't leak this type.
  pub fn new(device: vk::Device, memtype: Memtype, pagesize: vk::DeviceSize) -> Self {
    Self {
      device,
      memtype,
      pagesize,

      pages: Default::default(),
      bindings: Default::default(),
      free: Default::default(),
    }
  }

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
  fn allocate_page(&mut self, pagesize: vk::DeviceSize) -> Result<Block, Error> {
    let alloc_info = vk::MemoryAllocateInfo {
      sType: vk::STRUCTURE_TYPE_MEMORY_ALLOCATE_INFO,
      pNext: std::ptr::null(),
      allocationSize: pagesize,
      memoryTypeIndex: self.memtype.index,
    };

    let mut handle = vk::NULL_HANDLE;

    vk_check!(vk::AllocateMemory(self.device, &alloc_info, std::ptr::null(), &mut handle)).map_err(|_| Error::AllocError)?;
    assert!(handle != vk::NULL_HANDLE);

    let b = Block::new(handle, 0, pagesize, 0);
    self.free.insert(b, Node { prev: None, next: None });
    self.pages.insert(handle, Default::default());
    Ok(b)
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
    #[derive(Debug)]
    struct Group {
      b: usize,
      e: usize,
      block: Block,
    };

    let mut groups = Vec::new();
    let mut g = Group {
      b: 0,
      e: bindinfos.len(),
      block: Block::default(),
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
        groups.push(Group { b: g.b, e: g.e, block: b });
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
              groups.push(Group {
                b: g.b,
                e: g.b + c,
                block: b,
              });

              g.b = g.b + c;
              continue;
            }
          }
        }

        // Allocate a new page
        // if it fails, return the free block of every group to the table
        let b = self.allocate_page(vk::DeviceSize::max(size, self.pagesize))?;
        groups.push(Group { b: g.b, e: g.e, block: b });
        break;
      }
    }

    let mut blocks = Vec::with_capacity(bindinfos.len());
    let mut err = false;

    // bind the resources
    for g in groups.iter() {
      let infos = &bindinfos[g.b..g.e];

      // we always start at the beginning of the block, because by definition there has to be an occupied block or no block at all before
      let mut offset = g.block.beg;
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

    // insert free/occupied blocks
    for g in groups.iter() {
      let infos = &bindinfos[g.b..g.e];
      let blocks = &blocks[g.b..g.e];

      if let Some(n) = self.free.remove(&g.block) {
        // insert the first block always with the node of the group
        // prev reference will be good,
        // next reference will get fixed by subsequent blocks
        self.insert(BlockType::Occupied(blocks[0]), n.clone(), Some(infos[0].handle.get()));
        self.bindings.insert(infos[0].handle.get(), blocks[0]);

        // insert middle blocks, same as in the first one, next references will be fixed by subsequent blocks
        if blocks.len() > 2 {
          for i in 1..blocks.len() - 1 {
            self.insert(
              BlockType::Occupied(blocks[i]),
              Node {
                prev: Some(BlockType::Occupied(blocks[i - 1])),
                next: Some(BlockType::Occupied(blocks[i + 1])),
              },
              Some(infos[i].handle.get()),
            );
            self.bindings.insert(infos[i].handle.get(), blocks[i]);
          }
        }

        // last block
        if blocks.len() > 1 {
          let l = blocks.len() - 1;
          self.insert(
            BlockType::Occupied(blocks[l]),
            Node {
              prev: Some(BlockType::Occupied(blocks[l - 1])),
              next: n.next,
            },
            Some(infos[l].handle.get()),
          );
          self.bindings.insert(infos[l].handle.get(), blocks[l]);
        }

        // fix last block
        let remainder = Block::new(g.block.mem, blocks.last().unwrap().end, g.block.end, 0);
        if remainder.size() > 0 {
          self.insert(
            BlockType::Free(remainder),
            Node {
              prev: Some(BlockType::Occupied(*blocks.last().unwrap())),
              next: n.next,
            },
            None,
          );
        }
      }
    }

    debug_assert!(self.sanity_checks(), "bind sanity");
    Ok(())
  }

  /// Frees the allocated blocks of the specified handles
  ///
  /// Removes the mappings of all resources and merges the allocated and padding blocks back into the free list.
  ///
  /// Does NOT reshuffel the memory to maximize contiuous free blocks,
  /// because vulkan does not allow to rebind buffers/images.
  pub fn unbind(&mut self, handles: &[u64]) {
    let mut blocks = Vec::with_capacity(handles.len());
    for h in handles {
      if let Some(b) = self.bindings.remove(h) {
        blocks.push(b);
      }
    }

    blocks.sort_by_key(|b| b.beg);

    #[derive(Debug)]
    struct Group {
      block: Block,
      node: Node,
    }

    let mut groups: Vec<Group> = Vec::new();

    for b in blocks {
      let node = self.remove(BlockType::Occupied(b)).unwrap();

      match groups.iter_mut().find(|g| {
        if let Some(BlockType::Occupied(next)) = g.node.next {
          next == b
        } else {
          false
        }
      }) {
        Some(g) => {
          g.block.end = b.end;
          g.node.next = node.next;
        }
        None => groups.push(Group { block: b, node }),
      }
    }

    for g in groups.iter() {
      let mut freenode = g.node.clone();
      let mut freeblock = g.block;

      loop {
        match freenode.prev {
          Some(BlockType::Free(b)) => {
            if let Some(n) = self.remove(BlockType::Free(b)) {
              freeblock.beg = b.beg;
              freenode.prev = n.prev;
            }
          }
          _ => break,
        };
      }

      loop {
        match freenode.next {
          Some(BlockType::Free(b)) => {
            if let Some(n) = self.remove(BlockType::Free(b)) {
              freeblock.end = b.end;
              freenode.next = n.next;
            }
          }
          _ => break,
        };
      }

      if let Some(BlockType::Occupied(next)) = g.node.next {
        freeblock.end = next.beg + next.pad;
        if let Some(binding) = self.pages.get_mut(&next.mem).and_then(|p| p.remove(&next)) {
          let b = Block::new(next.mem, next.beg + next.pad, next.end, 0);
          self.bindings.insert(binding.handle, b);
          self.insert(BlockType::Occupied(b), binding.node, Some(binding.handle));
          freenode.next = Some(BlockType::Occupied(b));
        }
      }

      self.insert(BlockType::Free(freeblock), freenode, None);
    }

    debug_assert!(self.sanity_checks(), "unbind sanity");
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
    if let Some(n) = n.prev.and_then(|prev| self.get_node(prev)) {
      n.next = Some(b);
    }

    if let Some(n) = n.next.and_then(|next| self.get_node(next)) {
      n.prev = Some(b);
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

  fn remove(&mut self, b: BlockType) -> Option<Node> {
    if let Some(n) = match b {
      BlockType::Free(b) => self.free.remove(&b),
      BlockType::Occupied(b) => self.pages.get_mut(&b.mem).and_then(|p| p.remove(&b)).and_then(|b| Some(b.node)),
    } {
      if let Some(prevnode) = n.prev.and_then(|prev| self.get_node(prev)) {
        prevnode.next = n.next;
      }

      if let Some(nextnode) = n.next.and_then(|next| self.get_node(next)) {
        nextnode.prev = n.prev;
      }
      Some(n)
    } else {
      None
    }
  }

  /// Frees up pages with no allocation.
  pub fn free_unused(&mut self) {
    let empty = self
      .pages
      .iter()
      .filter_map(|(mem, blocks)| if blocks.is_empty() { Some(mem) } else { None });

    for mem in empty {
      vk::FreeMemory(self.device, *mem, std::ptr::null());

      if let Some(b) = self
        .free
        .iter()
        .find_map(|(b, _)| if b.mem == *mem { Some(b) } else { None })
        .cloned()
      {
        self.free.remove(&b);
      }
    }
  }

  /// Get the block of the specified resourcs.
  ///
  /// If the handle does not have a mapped block in this PageTable, returns None.
  pub fn get_mem(&self, handle: u64) -> Option<Block> {
    self.bindings.get(&handle).cloned()
  }

  /// Print stats abount all pages in yaml format
  pub fn print_stats(&self) -> String {
    let mut s = String::new();

    write!(s, "{}:\n", self.memtype);

    for (i, (mem, blocks)) in self.pages.iter().enumerate() {
      write!(s, "  Page{}({:x}):\n", i, mem);

      let mut blocks = blocks
        .iter()
        .map(|(b, _)| BlockType::Occupied(*b))
        .chain(
          self
            .free
            .iter()
            .filter_map(|(b, _)| if b.mem == *mem { Some(BlockType::Free(*b)) } else { None }),
        )
        .collect::<Vec<BlockType>>();
      blocks.sort_by_key(|b| b.get().beg);

      for b in blocks {
        write!(s, "    {}\n", b);
      }
    }

    //  write!(s, "    - SumAlloc   = {}\n", sum_alloc).unwrap();
    //  write!(s, "    - SumFree    = {}\n", sum_free).unwrap();
    //  write!(s, "    - SumPadding = {}\n", sum_paddings).unwrap();
    //  write!(s, "    - NAlloc = {}\n", n_alloc).unwrap();
    //  write!(s, "    - NFree  = {}\n", n_free).unwrap();
    s
  }

  fn sanity_checks(&mut self) -> bool {
    let mut allblocks = self
      .pages
      .iter()
      .map(|(mem, p)| p.keys().cloned())
      .flatten()
      .chain(self.free.iter().map(|(b, _)| *b))
      .fold(HashMap::new(), |mut acc, b| {
        acc.insert(b, false);
        acc
      });

    for (mem, p) in self.pages.iter() {
      // first block in this page
      let b = match p.iter().min_by_key(|(b, _)| b.beg).map(|(b, _)| b) {
        Some(b) => {
          // if the first block is not occupied its previous block has to be free and starting with 0
          if b.beg != 0 {
            match p.get(&b).and_then(|n| n.node.prev) {
              Some(BlockType::Free(b)) => BlockType::Free(b),
              _ => panic!("illegal first block {:?}", b),
            }
          } else {
            BlockType::Occupied(*b)
          }
        }
        // if no occupied block then there must be only a single free block
        None => return self.free.iter().filter(|(b, _)| b.mem == *mem).count() == 1,
      };

      let get_node = |bl| match bl {
        BlockType::Free(b) => self.free.get(&b),
        BlockType::Occupied(b) => p.get(&b).and_then(|x| Some(&x.node)),
      };

      // check that the previous block of b is the same as prevb
      let mut check = |prevb: &Option<BlockType>, b: &Option<BlockType>| {
        if let Some(b) = b {
          allblocks.entry(b.get()).and_modify(|x| *x = true);
          let n = match get_node(*b) {
            Some(n) => n,
            None => return false,
          };

          match n.prev {
            Some(prev) => prev.get() == prevb.unwrap().get(),
            None => prevb.is_none(),
          }
        } else {
          true
        }
      };

      let mut prevb = None;
      let mut b = Some(b);

      // traverse the node.next and check that the pointer blocks match
      while b.is_some() {
        if !check(&prevb, &b) {
          return false;
        }

        prevb = b;
        b = get_node(b.unwrap()).unwrap().next;
      }
    }

    allblocks.iter().any(|(_, b)| *b == true)
  }
}
