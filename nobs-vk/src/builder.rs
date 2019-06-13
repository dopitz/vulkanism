/// Trait for setting a builder for a vulkan struct
pub trait Buildable<T: Builder> {
  fn build() -> T {
    T::default()
  }
}

pub trait Builder: Default {
  type Target;
  fn raw(mut self, raw: Self::Target) -> Self;
}

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
