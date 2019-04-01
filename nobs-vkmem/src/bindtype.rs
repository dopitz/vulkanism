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


