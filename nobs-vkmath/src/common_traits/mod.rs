pub use std::ops::*;

pub trait SqrtAble {
  type Output;
  fn sqrt(self) -> Self::Output;
}
impl SqrtAble for f32 {
  type Output = f32;
  fn sqrt(self) -> f32 {
    f32::sqrt(self)
  }
}
impl SqrtAble for f64 {
  type Output = f64;
  fn sqrt(self) -> f64 {
    f64::sqrt(self)
  }
}

pub trait SinAble {
  type Output;
  fn sin(self) -> Self::Output;
  fn asin(self) -> Self::Output;
  fn cos(self) -> Self::Output;
  fn acos(self) -> Self::Output;
  fn tan(self) -> Self::Output;
  fn atan(self) -> Self::Output;
}
impl SinAble for f32 {
  type Output = f32;
  fn sin(self) -> f32 {
    f32::sin(self)
  }
  fn asin(self) -> f32 {
    f32::asin(self)
  }
  fn cos(self) -> f32 {
    f32::cos(self)
  }
  fn acos(self) -> f32 {
    f32::acos(self)
  }
  fn tan(self) -> f32 {
    f32::tan(self)
  }
  fn atan(self) -> f32 {
    f32::tan(self)
  }
}
impl SinAble for f64 {
  type Output = f64;
  fn sin(self) -> f64 {
    f64::sin(self)
  }
  fn asin(self) -> f64 {
    f64::asin(self)
  }
  fn cos(self) -> f64 {
    f64::cos(self)
  }
  fn acos(self) -> f64 {
    f64::acos(self)
  }
  fn tan(self) -> f64 {
    f64::tan(self)
  }
  fn atan(self) -> f64 {
    f64::tan(self)
  }
}

pub trait VecInto<T> {
  fn vec_into(self) -> T;
}

macro_rules! vec_into {
  ($from:ty, $to:ty) => {
    impl VecInto<$to> for $from {
      fn vec_into(self) -> $to {
        self as $to
      }
    }

    impl VecInto<$from> for $to {
      fn vec_into(self) -> $from {
        self as $from
      }
    }
  };
}

vec_into!(i8, i16);
vec_into!(i8, i32);
vec_into!(i8, i64);
vec_into!(i8, isize);
vec_into!(i8, u8);
vec_into!(i8, u16);
vec_into!(i8, u32);
vec_into!(i8, u64);
vec_into!(i8, usize);
vec_into!(i8, f32);
vec_into!(i8, f64);

vec_into!(i16, i32);
vec_into!(i16, i64);
vec_into!(i16, isize);
vec_into!(i16, u8);
vec_into!(i16, u16);
vec_into!(i16, u32);
vec_into!(i16, u64);
vec_into!(i16, usize);
vec_into!(i16, f32);
vec_into!(i16, f64);

vec_into!(i32, i64);
vec_into!(i32, isize);
vec_into!(i32, u8);
vec_into!(i32, u16);
vec_into!(i32, u32);
vec_into!(i32, u64);
vec_into!(i32, usize);
vec_into!(i32, f32);
vec_into!(i32, f64);

vec_into!(i64, isize);
vec_into!(i64, u8);
vec_into!(i64, u16);
vec_into!(i64, u32);
vec_into!(i64, u64);
vec_into!(i64, usize);
vec_into!(i64, f32);
vec_into!(i64, f64);

vec_into!(isize, u8);
vec_into!(isize, u16);
vec_into!(isize, u32);
vec_into!(isize, u64);
vec_into!(isize, usize);
vec_into!(isize, f32);
vec_into!(isize, f64);

vec_into!(u8, u16);
vec_into!(u8, u32);
vec_into!(u8, u64);
vec_into!(u8, usize);
vec_into!(u8, f32);
vec_into!(u8, f64);

vec_into!(u16, u32);
vec_into!(u16, u64);
vec_into!(u16, usize);
vec_into!(u16, f32);
vec_into!(u16, f64);

vec_into!(u32, u64);
vec_into!(u32, usize);
vec_into!(u32, f32);
vec_into!(u32, f64);

vec_into!(u64, usize);
vec_into!(u64, f32);
vec_into!(u64, f64);

vec_into!(usize, f32);
vec_into!(usize, f64);

vec_into!(f64, f32);

pub trait Identity {
  type Output;
  fn zero() -> Self::Output;
  fn one() -> Self::Output;
}

macro_rules! impl_identity {
  ( $( $t:ty ),* ) => (
    $(
      impl Identity for $t {
        type Output = $t;
        fn zero() -> $t {
          0 as $t
        }
        fn one() -> $t {
          1 as $t
        }
      }
    )*
  )
}
impl_identity!(i8, i16, i32, i64, u8, u16, u32, u64, f32, f64, usize);

pub trait ArithmeticOps<T>:
  Copy + Add<Output = T> + AddAssign<T> + Sub<Output = T> + SubAssign<T> + Mul<Output = T> + MulAssign<T> + Div<Output = T> + DivAssign<T>
{
}

macro_rules! impl_aritmetic_ops {
  ( $( $t:ty ),* ) => (
    $(
      impl ArithmeticOps<$t> for $t {
      }
    )*
  )
}
impl_aritmetic_ops!(i8, i16, i32, i64, u8, u16, u32, u64, f32, f64, usize);

pub trait VecTraits<T>: Default + Copy + ArithmeticOps<T> {}
macro_rules! impl_vec_traits {
  ( $( $t:ty ),* ) => (
    $(
      impl VecTraits<$t> for $t {
      }
    )*
  )
}
impl_vec_traits!(i8, i16, i32, i64, u8, u16, u32, u64, f32, f64, usize);

pub trait MatTraits<T>:
  Copy
  + Default
  + PartialEq
  + Identity<Output = T>
  + ArithmeticOps<T>
  + SinAble<Output = T>
  + Neg<Output = T>
  + VecTraits<T>
  + SqrtAble<Output = T>
{
}

impl MatTraits<f32> for f32 {}
impl MatTraits<f64> for f64 {}
