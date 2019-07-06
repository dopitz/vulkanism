
/// Enum defining resource types that can be bound
///
/// Handles that can be bound to memory are either of type vk::Buffer or vk::Image.
/// In both cases the handles are u64 typedefs.
/// The enum in used, so that we can submit buffer and image handles for binding to the Allocator in a uniform fashion,
/// whlile still being able to distinguish between them.
///
/// Submitting buffers along with images to the [Allocator](struct.Allocator.html) makes sence, when they use the same memory type
/// (which is the case for device local buffers and images)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Handle<T>
where
  T: Clone + Copy,
{
  Buffer(T),
  Image(T),
}

impl<T> Handle<T>
where
  T: Clone + Copy,
{
  /// Gets the underlying handle's value
  pub fn get(&self) -> T {
    match self {
      Handle::Image(h) => *h,
      Handle::Buffer(h) => *h,
    }
  }

  /// Convert the value of the handle without changing it's enum type
  pub fn map<U: Clone + Copy>(self, u: U) -> Handle<U> {
    match self {
      Handle::Image(_) => Handle::Image(u),
      Handle::Buffer(_) => Handle::Buffer(u),
    }
  }
}

