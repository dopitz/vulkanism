//! Automatic builder pattern generation for vulkan structs
//!
//! This module defines macros [vk_builder](../macro.vk_builder.html) and [vk_builder_into](../macro.vk_builder_into.html) for creating builder patterns for vulkan structs. The builder patterns are then invoked with an associated method on the vulkas struct.
//!
//! The crate does not implement any builders for any of the vulkan structs.
//!
//! Example implementation for 'vk::Rect2D':
//! ```ignore
//! #[macro_use] extern crate nobs_vk as vk;
//! use vk::builder::Buildable;
//!
//! // The builder for vulkan struct
//! pub struct Rect2DBuilder {
//!   rect: vk::Rect2D,
//! }
//!
//! // We need to define a default implementation
//! impl Default for Rect2DBuilder {
//!   fn default() -> Self {
//!     Self {
//!       rect: vk::Rect2D {
//!         offset: vk::Offset2D { x: 0, y: 0 },
//!         extent: vk::Extent2D { width: 0, height: 0 },
//!       },
//!     }
//!   }
//! }
//!
//! impl Rect2DBuilder {
//!   // ... do the builder stuff
//! }
//!
//! // Implements traits for convenient builder invocation
//! // Implements the AsRef<vk::Rect2D> trait
//! //vk_builder!(vk::Rect2D, Rect2DBuilder);
//!
//! // Additionaly implements the Into<vk::Rect2D> trait
//! vk_builder_into!(vk::Rect2D, Rect2DBuilder);
//!
//! // ... we can now instantiate a vk::Rect2D like so:
//! let rect = vk::Rect2D::build().into();
//! ```

/// Create a builder with an associated funcion `build()`
pub trait Buildable<T: Builder> {
  fn build() -> T {
    T::default()
  }
}

/// Trait all builders need to implement
///
/// This basically makes sure that we can create a builder from a preconfigured struct
pub trait Builder: Default {
  type Target;
  fn raw(self, raw: Self::Target) -> Self;
}

/// Implements [Buildable](trait.Buildable.html) for the target struct, [Builder](trait.Builder.html) and AsRef for the builder struct
///
/// See the [module level documentation](builder/index.html) for an examlpe.
#[macro_export]
macro_rules! vk_builder {
  ($target:ty, $builder:ident) => {
    $crate::vk_builder!($target, $builder, info);
  };
  ($target:ty, $builder:ident, $member:ident) => {
    impl $crate::builder::Buildable<$builder> for $target {}

    impl $crate::builder::Builder for $builder {
      type Target = $target;
      fn raw(mut self, raw: Self::Target) -> Self {
        self.$member = raw;
        self
      }
    }

    impl AsRef<$target> for $builder {
      fn as_ref(&self) -> &$target {
        &self.$member
      }
    }
  };
}

/// Additionally to [vk_builder](macro.vk_builder.html) implements Into for the builder struct
///
/// See the [module level documentation](builder/index.html) for an examlpe.
#[macro_export]
macro_rules! vk_builder_into {
  ($target:ty, $builder:ident) => {
    $crate::vk_builder_into!($target, $builder, info);
  };
  ($target:ty, $builder:ident, $member:ident) => {
    $crate::vk_builder!($target, $builder, $member);

    impl Into<$target> for $builder {
      fn into(self) -> $target {
        self.$member
      }
    }
  };
}
