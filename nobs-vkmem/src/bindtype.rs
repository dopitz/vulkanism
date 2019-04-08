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
}


