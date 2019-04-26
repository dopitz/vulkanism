use crate::common_traits::*;

#[repr(C)]
#[derive(Default, Clone, Copy, Debug)]
pub struct Vec2<T> {
  pub x: T,
  pub y: T,
}

// Constructors
impl<T: Copy> Vec2<T> {
  pub fn repeat(v: T) -> Self {
    Self { x: v, y: v }
  }

  pub fn new(x: T, y: T) -> Self {
    Self { x, y }
  }

  pub fn from<U: VecInto<T>>(v: Vec2<U>) -> Self {
    Self {
      x: v.x.vec_into(),
      y: v.y.vec_into(),
    }
  }

  pub fn into<U>(self) -> Vec2<U>
  where
    T: VecInto<U>,
  {
    Vec2::<U> {
      x: self.x.vec_into(),
      y: self.y.vec_into(),
    }
  }
}

// Compare
impl<T: PartialEq> PartialEq for Vec2<T> {
  fn eq(&self, other: &Self) -> bool {
    self.x == other.x && self.y == other.y
  }
}

// Element Access
impl<T> Index<usize> for Vec2<T> {
  type Output = T;

  fn index(&self, i: usize) -> &T {
    match i {
      0 => &self.x,
      1 => &self.y,
      n => panic!("vec2 used with index {}", n),
    }
  }
}
impl<T> IndexMut<usize> for Vec2<T> {
  fn index_mut(&mut self, i: usize) -> &mut T {
    match i {
      0 => &mut self.x,
      1 => &mut self.y,
      n => panic!("vec2 used with index {}", n),
    }
  }
}

// Arithmetic Ops
impl<T: Neg<Output = T>> Neg for Vec2<T> {
  type Output = Vec2<T>;
  fn neg(self) -> Self {
    Self { x: -self.x, y: -self.y }
  }
}

impl<T: VecTraits<T>> Add<Vec2<T>> for Vec2<T> {
  type Output = Vec2<T>;
  fn add(self, other: Self) -> Self {
    Self {
      x: self.x + other.x,
      y: self.y + other.y,
    }
  }
}
impl<T: VecTraits<T>> AddAssign<Vec2<T>> for Vec2<T> {
  fn add_assign(&mut self, other: Self) {
    self.x += other.x;
    self.y += other.y;
  }
}

impl<T: VecTraits<T>> Sub<Vec2<T>> for Vec2<T> {
  type Output = Vec2<T>;
  fn sub(self, other: Self) -> Self {
    Self {
      x: self.x - other.x,
      y: self.y - other.y,
    }
  }
}
impl<T: VecTraits<T>> SubAssign<Vec2<T>> for Vec2<T> {
  fn sub_assign(&mut self, other: Self) {
    self.x -= other.x;
    self.y -= other.y;
  }
}

impl<T: VecTraits<T>> Mul<Vec2<T>> for Vec2<T> {
  type Output = Vec2<T>;
  fn mul(self, other: Self) -> Self {
    Self {
      x: self.x * other.x,
      y: self.y * other.y,
    }
  }
}
impl<T: VecTraits<T>> Mul<T> for Vec2<T> {
  type Output = Vec2<T>;
  fn mul(self, other: T) -> Self {
    Self {
      x: self.x * other,
      y: self.y * other,
    }
  }
}
impl<T: VecTraits<T>> MulAssign<Vec2<T>> for Vec2<T> {
  fn mul_assign(&mut self, other: Self) {
    self.x *= other.x;
    self.y *= other.y;
  }
}

impl<T: VecTraits<T>> Div<Vec2<T>> for Vec2<T> {
  type Output = Vec2<T>;
  fn div(self, other: Self) -> Self {
    Self {
      x: self.x / other.x,
      y: self.y / other.y,
    }
  }
}
impl<T: VecTraits<T>> Div<T> for Vec2<T> {
  type Output = Vec2<T>;
  fn div(self, other: T) -> Self {
    Self {
      x: self.x / other,
      y: self.y / other,
    }
  }
}
impl<T: VecTraits<T>> DivAssign<Vec2<T>> for Vec2<T> {
  fn div_assign(&mut self, other: Self) {
    self.x /= other.x;
    self.y /= other.y;
  }
}

// Vector Ops
impl<T: PartialOrd + VecTraits<T>> Vec2<T> {
  pub fn min(a: Self, b: Self) -> Self {
    Self {
      x: if a.x < b.x { a.x } else { b.x },
      y: if a.y < b.y { a.y } else { b.y },
    }
  }

  pub fn max(a: Self, b: Self) -> Self {
    Self {
      x: if a.x > b.x { a.x } else { b.x },
      y: if a.y > b.y { a.y } else { b.y },
    }
  }

  pub fn clamp(v: Self, lo: Self, hi: Self) -> Self {
    let mut v = v;
    if v.x < lo.x {
      v.x = lo.x;
    }
    if v.x > hi.x {
      v.x = hi.x;
    }

    if v.y < lo.y {
      v.y = lo.y;
    }
    if v.y > hi.y {
      v.y = hi.y;
    }
    v
  }
}

impl<T: VecTraits<T>> Vec2<T> {
  pub fn dot(a: Self, b: Self) -> T {
    a.x * b.x + a.y * b.y
  }
}

impl<T: VecTraits<T> + SqrtAble<Output = T>> Vec2<T> {
  pub fn len(v: Self) -> T {
    Self::dot(v, v).sqrt()
  }
}

impl<T: VecTraits<T> + SqrtAble<Output = T>> Vec2<T> {
  pub fn normalize(v: Self) -> Self {
    v / Self::len(v)
  }
}

impl<T: VecTraits<T>> Vec2<T> {
  pub fn lerp(a: Self, b: Self, t: T) -> Self {
    a + (b - a) * t
  }
}

pub type Vec2f = Vec2<f32>;
pub type Vec2d = Vec2<f64>;
pub type Vec2i = Vec2<i32>;
pub type Vec2u = Vec2<u32>;
pub type Vec2s = Vec2<usize>;

#[macro_export]
macro_rules! vec2 {
  () => {
    crate::vec2::Vec2::default()
  };
  ($x:expr) => {
    crate::vec2::Vec2::new($x, $x)
  };
  ($x:expr, ..) => {{
    let mut v = crate::vec2::Vec2::default();
    v.x = $x;
    v
  }};
  ($x:expr, $y:expr) => {
    crate::vec2::Vec2::new($x, $y)
  };
}

#[cfg(test)]
mod tests {
  use crate::vec2::*;

  #[test]
  fn ctor() {
    let d = Vec2f::default();
    let r = Vec2f::repeat(1.2);
    let n = Vec2::new(0.5, 1.3);
    assert_eq!(d.x, 0.0);
    assert_eq!(d.y, 0.0);
    assert_eq!(r.x, 1.2);
    assert_eq!(r.y, 1.2);
    assert_eq!(n.x, 0.5);
    assert_eq!(n.y, 1.3);
  }

  #[test]
  fn ctor_macro() {
    let a = vec2!(1.2, ..);
    let b = vec2!(1.2);
    let c = vec2!(0.5, 1.3);
    assert_eq!(a.x, 1.2);
    assert_eq!(a.y, 0.0);
    assert_eq!(b.x, 1.2);
    assert_eq!(b.y, 1.2);
    assert_eq!(c.x, 0.5);
    assert_eq!(c.y, 1.3);
  }

  #[test]
  fn eq() {
    let a = vec2!(1.2);
    let b = vec2!(0.5, 1.3);
    let c = vec2!(0.5, 1.3);
    assert_eq!(a == b, false);
    assert_eq!(a != b, true);
    assert_eq!(b == c, true);
  }

  #[test]
  fn add() {
    let a = vec2!(1.2);
    let b = vec2!(0.5, 1.3);
    let mut c = a + b;
    assert_eq!(c.x, 1.7);
    assert_eq!(c.y, 2.5);

    c += b;
    assert_eq!(c.x, 2.2);
    assert_eq!(c.y, 3.8);
  }

  #[test]
  fn sub() {
    let a = vec2!(1.2);
    let b = vec2!(0.5, 1.2);
    let mut c = a - b;
    assert_eq!(c.x, 0.7);
    assert_eq!(c.y, 0.0);

    c -= c;
    assert_eq!(c.x, 0.0);
    assert_eq!(c.y, 0.0);
  }

  #[test]
  fn mul() {
    let a = vec2!(1);
    let b = vec2!(5, 3);
    let mut c = a * b;
    assert_eq!(c.x, 5);
    assert_eq!(c.y, 3);

    c *= c;
    assert_eq!(c.x, 25);
    assert_eq!(c.y, 9);

    let b = b * 2;
    assert_eq!(b.x, 10);
    assert_eq!(b.y, 6);
  }

  #[test]
  fn div() {
    let a = vec2!(10);
    let b = vec2!(5, 3);
    let mut c = a / b;
    assert_eq!(c.x, 2);
    assert_eq!(c.y, 3);

    c /= vec2!(2, 1);
    assert_eq!(c.x, 1);
    assert_eq!(c.y, 3);

    let a = a / 2;
    assert_eq!(a.x, 5);
    assert_eq!(a.y, 5);
  }

  #[test]
  fn minmax() {
    let a = vec2!(4.0);
    let b = vec2!(5.0, 3.0);
    let c = Vec2::min(a, b);
    assert_eq!(c.x, 4.0);
    assert_eq!(c.y, 3.0);
    let c = Vec2::max(a, b);
    assert_eq!(c.x, 5.0);
    assert_eq!(c.y, 4.0);
  }

  #[test]
  fn len() {
    assert_eq!(Vec2f::len(vec2!(2, 0).into()), 2.0);
    assert_eq!(Vec2::len(vec2!(2.0, 1.0)), f32::sqrt(5.0));
    assert_eq!(Vec2::len(vec2!(2.0, 2.0)), f32::sqrt(8.0));
    assert!(f32::abs(Vec2::len(Vec2::normalize(vec2!(2.0, 2.0))) - 1.0) < 0.0001);
  }

  #[test]
  fn cast() {
    let i = vec2!(1, 2);
    let f = vec2!(1.1, 2.2);

    let ii = i * f.into();
    let fi = f * i.into();

    assert_eq!(ii.x, 1);
    assert_eq!(ii.y, 4);

    assert_eq!(fi.x, 1.1);
    assert_eq!(fi.y, 4.4);
  }
}
