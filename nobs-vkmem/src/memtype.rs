/// Memory type that is returnd by `vk::Get*MemoryRequirements` functions
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Memtype {
  pub index: u32,
  pub linear: bool,
}

impl std::fmt::Display for Memtype {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(f, "memtype({}, {})", self.index, if self.linear { "linear" } else { "non-linear" })
  }
}
