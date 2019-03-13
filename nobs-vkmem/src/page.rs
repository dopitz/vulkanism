use std::collections::{BTreeSet, HashMap, HashSet};
use std::fmt::Write;

use crate::block::Block;
use crate::Error;
use crate::Handle;
use vk;

/// Type of a [Block](../block/struct.Block.html)
#[derive(Debug)]
enum BlockType {
  Free,
  Padded,
  Alloc,
}

/// Internal bind info used only by PageTable
///
/// Implements conversion from the public [BindInfo](../struct.BindInfo.html).
/// The `size` field may be smaller than the `requirements.size`, in case the public BindInfo specified it.
/// Otherwise these two sizes will be equal.
#[derive(Debug, Clone, Copy)]
pub struct BindInfo {
  pub handle: Handle<u64>,
  pub size: vk::DeviceSize,
  pub requirements: vk::MemoryRequirements,
  pub linear: bool,
}

impl BindInfo {
  /// Create BindInfo from the public [BindInfo](../struct.BindInfo.html).
  ///
  /// Reads the buffer/image requirements from vulkan.
  /// If `info.size` specifies a valid size `self.size` will be initialized with this size, otherwise the requirement's size will be used.
  pub fn new(device: vk::Device, info: &crate::BindInfo) -> Self {
    let mut requirements = unsafe { std::mem::uninitialized() };
    match info.handle {
      Handle::Image(i) => vk::GetImageMemoryRequirements(device, i, &mut requirements),
      Handle::Buffer(b) => vk::GetBufferMemoryRequirements(device, b, &mut requirements),
    }
    let handle = info.handle;
    let size = match info.size {
      Some(size) => size,
      None => requirements.size,
    };
    let linear = info.linear;

    Self {
      handle,
      size,
      requirements,
      linear,
    }
  }
}

/// Strategy how resources are bound in the [Allocator](struct.Allocator.html)
#[derive(Debug, Clone, Copy)]
pub enum BindType {
  /// The allocator may split up groups of resources.
  /// As a consequence, not all resources will be bound to a continuous block of memory.
  /// This makes the best usage of memory space.
  Scatter,
  /// The allocator is forced to bind all resources to a single continuous block of memory.
  /// If no such block exists a new page will be allocated.
  Block,
  /// Allocates the resources on a NEW private page, with the exact size that is needed.
  /// If one or more of the resources allocated with this type is [unbound](struct.Allocator.html#method.destroy) again,
  /// the freed space on this page is free to be used by newly created resources.
  Minipage,
}

/// A PageTable manages allocations of the same memory type.
///
/// Memory is managed in Blocks of different [type](enum.BlockType.html).
/// The free and padding blocks are sorted globally by decreasing size and ascending address.
/// Allocated blocks are mapped by their respective resource handle.
pub struct PageTable {
  device: vk::Device,

  memtype: u32,
  pagesize: vk::DeviceSize,

  begin_linear: vk::DeviceSize,
  free: BTreeSet<Block>,
  padding: BTreeSet<Block>,
  allocations: HashMap<u64, Block>,
}

impl Drop for PageTable {
  fn drop(&mut self) {
    let mut pages = HashSet::new();

    for b in self.free.iter() {
      pages.insert(b.mem);
    }

    for (_, b) in self.allocations.iter() {
      pages.insert(b.mem);
    }

    for &p in pages.iter() {
      vk::FreeMemory(self.device, p, std::ptr::null());
    }
  }
}

impl PageTable {
  /// Creates a new page table with the desired page size.
  ///
  /// We do not need to check for the minimum page size, since [Allocator](../struct.Allocator.html) already does that, and we don't leak this type.
  pub fn new(device: vk::Device, memtype: u32, pagesize: vk::DeviceSize) -> Self {
    Self {
      device,
      memtype,
      pagesize,
      begin_linear: 0,
      free: Default::default(),
      padding: Default::default(),
      allocations: Default::default(),
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
      memoryTypeIndex: self.memtype,
    };

    let mut handle = vk::NULL_HANDLE;

    vk_check!(vk::AllocateMemory(self.device, &alloc_info, std::ptr::null(), &mut handle)).map_err(|_| Error::AllocError)?;
    assert!(handle != vk::NULL_HANDLE);

    Ok(Block::with_size(0, self.pagesize, handle))
  }

  /// Compute blocks from [BindInfos](struct.BindInfo.html)
  ///
  /// Requires that `bindinfos.len() == blocks.len()`.
  ///
  /// Sets the begin and end field in every block. The offsets are computed so that every block is aligned correctly w/r to their BindInfo's reqirements.
  /// The block's begin and end are computed to base address 0, e.g. the first block starts with `begin == 0`.
  /// Returns the reqired alignment of the first block and the required total size of all blocks.
  fn compute_blocks(bindinfos: &[BindInfo], blocks: &mut [Block]) -> (vk::DeviceSize, vk::DeviceSize) {
    // use the largest alignment for all resources
    let alignment = bindinfos
      .iter()
      .fold(0, |align, i| vk::DeviceSize::max(align, i.requirements.alignment));

    let mut prev = Block::new(0, 0, 0);
    for (block, info) in blocks.iter_mut().zip(bindinfos.iter()) {
      let begin = match prev.end % info.requirements.alignment {
        0 => prev.end,
        modulo => prev.end + info.requirements.alignment - modulo,
      };

      *block = Block::with_size(begin, info.size, 0);
      prev = *block;
    }

    (alignment, prev.end)
  }

  /// Finds the smallest block in `blocks`, that is still than the specified size after padding to requested alignment.
  fn smallest_fit(blocks: &BTreeSet<Block>, alignment: vk::DeviceSize, size: vk::DeviceSize, begin_linear: vk::DeviceSize, linear: bool) -> Option<Block> {
    if linear {
    blocks
      .range(Block::with_size(0, size + alignment, 0)..)
      .filter(|b| b.size_aligned(alignment) >= size)
      .find(|b| b.begin >= begin_linear)
      .cloned()
    }
    else {
    blocks
      .range(Block::with_size(0, size + alignment, 0)..)
      .filter(|b| b.size_aligned(alignment) >= size)
      .rev()
      .find(|b| b.begin <= begin_linear)
      .cloned()
    }
  }

  /// Moves blocks to the destination.
  ///
  /// When blocks are computed with [compute_blocks](struct.PageTable.html#method.compute_blocks) they start at (begin == 0).
  /// This function shifts the blocks begin and end to the beginning of `dst` with the requested `alignment`.
  fn move_blocks(blocks: &mut [Block], dst: Block, alignment: vk::DeviceSize) -> Block {
    if blocks.is_empty() {
      return dst;
    }

    let offset = dst.begin + dst.size_aligned(alignment) - dst.size();
    let mem = dst.mem;

    for b in blocks.iter_mut() {
      b.begin += offset;
      b.end += offset;
      b.mem = mem;
    }

    assert!(blocks.last().unwrap().end <= dst.end);
    Block::new(blocks.last().unwrap().end, dst.end, dst.mem)
  }

  /// Binds handles from `bindinfos` to their corresponding `blocks`.
  ///
  /// Fails, if the vulkan command to bind the memory does not return successfully.
  ///
  /// Inserts padding between every block automatically.
  /// Inserts additional `paddings` (at the begin or end of a used up block).
  /// Removes `used` blocks from the free list.
  /// Inserts `free` blocks to the free list.
  fn bind_blocks(
    &mut self,
    bindinfos: &[BindInfo],
    blocks: &[Block],
    paddings: &[Block],
    used: &[Block],
    free: &[Block],
  ) -> Result<(), Error> {
    for (b, h) in blocks.iter().zip(bindinfos.iter().map(|i| i.handle)) {
      match h {
        Handle::Buffer(h) => vk_check!(vk::BindBufferMemory(self.device, h, b.mem, b.begin)),
        Handle::Image(h) => vk_check!(vk::BindImageMemory(self.device, h, b.mem, b.begin)),
      }
      .map_err(|_| Error::BindMemoryFailed)?;
    }

    for (i, b) in blocks.iter().enumerate() {
      self.allocations.insert(bindinfos[i].handle.get(), *b);
    }

    // we infer paddings by comparing two consecutive allocated blocks
    for (this, next) in blocks.iter().zip(blocks.iter().skip(1)) {
      if this.mem == next.mem && this.end != next.begin {
        self.padding.insert(Block::new(this.end, next.begin, this.mem));
      }
    }

    // additional paddings (at the beggining and end of a used block)
    for pad in paddings {
      self.padding.insert(*pad);
    }

    // block that was previously free is now used up
    for u in used {
      self.free.remove(&u);
    }

    // remainder of a free block
    for f in free {
      self.free.insert(*f);
    }

    Ok(())
  }

  /// Finds free blocks to which resources can be bound
  ///
  /// Tries to fit all resources into a single free block, if there is no such block, splits them up.
  /// If a valid free block is found,
  ///  - sets up the respective subrange of `blocks` with [move_blocks](struct.PageTable.html#method.move_blocks)
  ///  - adds this valid block to `used` and removes it from self.free
  ///  - adds padding or free block at the beginning and and of the used up space and adds them to `paddings` / `free`
  ///
  /// If the recursion gets to a subrange of length 1 and still no valid free block is found
  fn bind_recursive(
    &mut self,
    bindinfos: &[BindInfo],
    blocks: &mut [Block],
    paddings: &mut Vec<Block>,
    used: &mut Vec<Block>,
    free: &mut Vec<Block>,
  ) -> Result<(), Error> {
    let (alignment, size) = Self::compute_blocks(bindinfos, blocks);

    // if we can find a matching block for this subrange
    // setup blocks, remove and store the free block and insert a paddings if needed
    if let Some(best) = Self::smallest_fit(&self.free, alignment, size, self.begin_linear, false) {
      Self::move_blocks(blocks, best, alignment);
      self.free.remove(&best);

      // if the best block is not aligned, insert padding
      if blocks[0].begin != best.begin {
        paddings.push(Block::new(best.begin, blocks[0].begin, best.mem));
      }
      // insert the remaining free space as free block,
      // if the block is too smal insert it as padding
      let end = blocks.last().unwrap().end;
      if end != best.end {
        let rest = Block::new(end, best.end, best.mem);
        if rest.size_aligned(256) < 256 {
          paddings.push(rest);
        } else {
          free.push(rest);
        }
      }
      // we used up a free block
      used.push(best);
      return Ok(());
    }

    // if not we subdevide the rage further
    // in case we can't we immediately return with an error
    if bindinfos.len() == 1 {
      Err(Error::OutOfMemory)?;
    }

    // if there are at least 2 buffers remaining we can split the rages in half and try again
    let split = bindinfos.len() / 2;
    if let Err(e) = self
      .bind_recursive(&bindinfos[0..split], &mut blocks[split..], paddings, used, free)
      .and(self.bind_recursive(&bindinfos[split..], &mut blocks[split..], paddings, used, free))
    {
      // reinsert the used blocks again
      for u in used.iter() {
        self.free.insert(*u);
      }
      Err(e)?;
    }
    Ok(())
  }

  /// Binds all resourcses without
  ///
  /// see [BindType::Scatter](enum.BindType.html).
  ///
  /// Fails of no trivially distribution of resources to free blocks is found (does not solve the bin-packing).
  fn bind_scatter(&mut self, bindinfos: &[BindInfo], blocks: &mut [Block]) -> Result<(), Error> {
    let mut paddings = Vec::new();
    let mut used = Vec::new();
    let mut free = Vec::new();
    loop {
      paddings.clear();
      used.clear();
      free.clear();
      match self.bind_recursive(bindinfos, blocks, &mut paddings, &mut used, &mut free) {
        Ok(()) => break,
        Err(_) => {
          let page = self.allocate_page(self.pagesize)?;
          self.free.insert(page)
        }
      };
    }

    self.bind_blocks(bindinfos, &blocks, &paddings, &used, &free)
  }

  /// Binds all resourcses to a continuous block in memory
  ///
  /// see [BindType::Block](enum.BindType.html).
  ///
  /// The difference to [bind](struct.PageTable.html#method.bind_scatter) is,
  /// that if the first attempt of [bind_recursive](struct.PageTable.html#method.bind_recursive)
  fn bind_block(&mut self, bindinfos: &[BindInfo], blocks: &mut [Block]) -> Result<(), Error> {
    let mut paddings = Vec::new();
    let mut used = Vec::new();
    let mut free = Vec::new();

    // try to bind, we allocate if the number of used blocks is larger 1 (or OutOfMemory)
    if match self.bind_recursive(bindinfos, blocks, &mut paddings, &mut used, &mut free) {
      Err(_) => true,
      Ok(_) => used.len() > 1,
    } {
      let page = self.allocate_page(self.pagesize)?;
      self.free.insert(page);

      paddings.clear();
      used.clear();
      free.clear();
      self.bind_recursive(bindinfos, blocks, &mut paddings, &mut used, &mut free)?;

      if used.len() > 1 {
        Err(Error::OversizedBlock)?;
      }
    }

    self.bind_blocks(bindinfos, &blocks, &paddings, &used, &free)
  }

  /// Binds all resourcses to their own page
  ///
  /// see [BindType::Block](enum.BindType.html).
  ///
  /// This will always allocate exactly as much memory as it is needed to bind all resources in `bindinfos`
  fn bind_minipage(&mut self, bindinfos: &[BindInfo], blocks: &mut [Block]) -> Result<(), Error> {
    let page = {
      // since we are allocating memory, we have to make sure, that size - offset == alignment
      let lastblock = blocks.last_mut().unwrap();
      let lastinfo = bindinfos.last().unwrap();
      if lastblock.size() != lastinfo.requirements.size {
        lastblock.end = lastblock.begin + lastinfo.requirements.size;
      }

      self.allocate_page(lastblock.end)?
    };

    Self::move_blocks(blocks, page, 1);

    self.bind_blocks(bindinfos, &blocks, &[], &[], &[])
  }

  /// Bind resources to memory
  ///
  /// See [BindType::Block](enum.BindType.html).
  pub fn bind(&mut self, bindinfos: &[BindInfo], bindtype: BindType) -> Result<(), Error> {
    if bindinfos.is_empty() {
      return Ok(());
    }

    if bindinfos.iter().any(|i| self.allocations.contains_key(&i.handle.get())) {
      Err(Error::AlreadyBound)?;
    }
    if bindinfos.iter().any(|i| i.requirements.memoryTypeBits & (1 << self.memtype) == 0) {
      Err(Error::InvalidMemoryType)?;
    }

    let mut blocks = Vec::with_capacity(bindinfos.len());
    blocks.resize(bindinfos.len(), Block::new(0, 0, 0));
    Self::compute_blocks(bindinfos, &mut blocks);

    match bindtype {
      BindType::Scatter => self.bind_scatter(bindinfos, &mut blocks),
      BindType::Block => self.bind_block(bindinfos, &mut blocks),
      BindType::Minipage => self.bind_minipage(bindinfos, &mut blocks),
    }
  }

  /// Collects all blocks in `src` that overlap with at least one block in `dst`
  ///
  /// Overlapping Blocks in src are deleted.
  /// Blocks are merged with dst, so that there are no overlapping blocks in dst.
  fn merge_blocks(dst: &mut Vec<Block>, src: &mut BTreeSet<Block>) {
    loop {
      let overlap = src
        .iter()
        .filter(|b| dst.is_empty() || dst.iter().any(|a| Block::overlapping(&a, &b)))
        .cloned()
        .collect::<Vec<_>>();

      if overlap.is_empty() {
        break;
      }

      // a single block might overlap with at most 2 blocks in set
      let mut delete = Vec::with_capacity(2);
      for b in overlap.iter() {
        src.remove(b);

        delete.clear();
        let mut merged = *b;
        for (i, b) in dst.iter().enumerate() {
          if let Some(merge) = Block::merge(&merged, b) {
            delete.push(i);
            merged = merge;
          }
        }

        debug_assert!(delete.len() <= 2);

        match delete.len() {
          0 => dst.push(merged),
          1 => dst[delete[0]] = merged,
          2 => {
            dst[delete[0]] = merged;
            dst.remove(delete[1]);
          }
          _ => (),
        }
      }
    }
  }

  /// Frees the allocated blocks of the specified handles
  ///
  /// Removes the mappings of all resources and merges the allocated and padding blocks back into the free list.
  ///
  /// Does NOT reshuffel the memory to maximize contiuous free blocks,
  /// because vulkan does not allow to rebind buffers/images.
  pub fn unbind(&mut self, handles: &[u64]) {
    // merge_into takes care of overlapping allocation of handles as well
    let mut free = Vec::new();
    Self::merge_blocks(
      &mut free,
      &mut handles
        .iter()
        .filter_map(|h| self.allocations.get(h))
        .cloned()
        .collect::<BTreeSet<_>>(),
    );

    // delete the allocations
    for h in handles {
      self.allocations.remove(h);
    }

    // merge the freed allocations with overlapping paddings and free blocks
    Self::merge_blocks(&mut free, &mut self.padding);
    Self::merge_blocks(&mut free, &mut self.free);

    for b in free {
      self.free.insert(b);
    }
  }

  /// Collect blocks sorted by pages and BlockType
  fn collect_pages(&self) -> HashMap<vk::DeviceMemory, Vec<(BlockType, Block)>> {
    self
      .free
      .iter()
      .map(|b| (BlockType::Free, *b))
      .chain(self.padding.iter().map(|b| (BlockType::Padded, *b)))
      .chain(self.allocations.iter().map(|(_, b)| (BlockType::Alloc, *b)))
      .fold(HashMap::new(), |mut acc, b| {
        acc.entry(b.1.mem).or_insert(Vec::new()).push(b);
        acc
      })
  }

  /// Frees up pages with no allocation.
  pub fn free_unused(&mut self) {
    let empty = self
      .collect_pages()
      .iter()
      .filter_map(|(_, blocks)| match blocks.len() == 1 {
        true => match blocks[0].0 {
          BlockType::Free => Some(blocks[0].1),
          _ => None,
        },
        false => None,
      })
      .collect::<Vec<_>>();

    for b in empty {
      vk::FreeMemory(self.device, b.mem, std::ptr::null());
      self.free.remove(&b);
    }
  }

  /// Get the block of the specified resourcs.
  ///
  /// If the handle does not have a mapped block in this PageTable, returns None.
  pub fn get_mem(&self, handle: u64) -> Option<Block> {
    self.allocations.get(&handle).cloned()
  }

  /// Print stats abount all pages in yaml format
  pub fn print_stats(&self) -> String {
    let mut s = String::new();
    for (i, (mem, blocks)) in self.collect_pages().iter_mut().enumerate() {
      write!(s, "MemType: {}\n", self.memtype).unwrap();
      blocks.sort_by_key(|b| b.1.begin);

      let mut sum_alloc = 0;
      let mut sum_free = 0;
      let mut sum_paddings = 0;
      let mut n_alloc = 0;
      let mut n_free = 0;

      write!(s, "  - Page{} ({:x}):\n", i, mem).unwrap();
      for (t, b) in blocks.iter() {
        write!(s, "    - {:?}{{begin: {}, end: {}}}\n", t, b.begin, b.end).unwrap();

        match t {
          BlockType::Alloc => {
            sum_alloc += b.size();
            n_alloc += 1;
          }
          BlockType::Free => {
            sum_free += b.size();
            n_free += 1;
          }
          BlockType::Padded => {
            sum_paddings += b.size();
          }
        }
      }

      write!(s, "    - SumAlloc   = {}\n", sum_alloc).unwrap();
      write!(s, "    - SumFree    = {}\n", sum_free).unwrap();
      write!(s, "    - SumPadding = {}\n", sum_paddings).unwrap();
      write!(s, "    - NAlloc = {}\n", n_alloc).unwrap();
      write!(s, "    - NFree  = {}\n", n_free).unwrap();
    }
    s
  }
}
