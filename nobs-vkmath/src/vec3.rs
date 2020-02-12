use crate::common_traits::*;
use crate::vec2::Vec2;

#[repr(C)]
#[derive(Default, Clone, Copy, Debug)]
pub struct Vec3<T> {
  pub x: T,
  pub y: T,
  pub z: T,
}

// Constructors
impl<T: Copy> Vec3<T> {
  pub fn repeat(v: T) -> Self {
    Self { x: v, y: v, z: v }
  }

  pub fn new(x: T, y: T, z: T) -> Self {
    Self { x, y, z }
  }

  pub fn from<U: VecInto<T>>(v: Vec3<U>) -> Self {
    Self {
      x: v.x.vec_into(),
      y: v.y.vec_into(),
      z: v.z.vec_into(),
    }
  }

  pub fn into<U>(self) -> Vec3<U>
  where
    T: VecInto<U>,
  {
    Vec3::<U> {
      x: self.x.vec_into(),
      y: self.y.vec_into(),
      z: self.z.vec_into(),
    }
  }

  pub fn xy(&self) -> Vec2<T> {
    vec2!(self.x, self.y)
  }

  pub fn from_xy(v: Vec2<T>, z: T) -> Self {
    Self::new(v.x, v.y, z)
  }

  pub fn map_x<F: Fn(&Self) -> T>(mut self, f: F) -> Self {
    self.x = f(&self);
    self
  }
  pub fn map_y<F: Fn(&Self) -> T>(mut self, f: F) -> Self {
    self.y = f(&self);
    self
  }
  pub fn map_z<F: Fn(&Self) -> T>(mut self, f: F) -> Self {
    self.z = f(&self);
    self
  }
  pub fn map_xy<F: Fn(&Self) -> Vec2<T>>(mut self, f: F) -> Self {
    let v = f(&self);
    self.x = v.x;
    self.y = v.y;
    self
  }
  pub fn map<F: Fn(&Self) -> Self>(self, f: F) -> Self {
    f(&self)
  }
  pub fn map_into<U, F: Fn(&Self) -> U>(self, f: F) -> U {
    f(&self)
  }
}

impl<T: Identity<Output = T>> Vec3<T> {
  pub fn zero() -> Self {
    Self { x: T::zero(), y: T::zero(), z: T::zero() }
  }
}

// Compare
impl<T: PartialEq> PartialEq for Vec3<T> {
  fn eq(&self, other: &Self) -> bool {
    self.x == other.x && self.y == other.y && self.z == other.z
  }
}
impl<T: Eq> Eq for Vec3<T> {}

// Element Access
impl<T> Index<usize> for Vec3<T> {
  type Output = T;

  fn index(&self, i: usize) -> &T {
    match i {
      0 => &self.x,
      1 => &self.y,
      2 => &self.z,
      n => panic!("vec3 used with index {}", n),
    }
  }
}
impl<T> IndexMut<usize> for Vec3<T> {
  fn index_mut(&mut self, i: usize) -> &mut T {
    match i {
      0 => &mut self.x,
      1 => &mut self.y,
      2 => &mut self.z,
      n => panic!("vec3 used with index {}", n),
    }
  }
}

// Arithmetic Ops
impl<T: Neg<Output = T>> Neg for Vec3<T> {
  type Output = Vec3<T>;
  fn neg(self) -> Self {
    Self {
      x: -self.x,
      y: -self.y,
      z: -self.z,
    }
  }
}
impl<T: PartialOrd + Identity<Output = T> + Neg<Output = T>> Vec3<T> {
  pub fn abs(self) -> Self {
    Self {
      x: if self.x < T::zero() {-self.x} else {self.x},
      y: if self.y < T::zero() {-self.y} else {self.y},
      z: if self.z < T::zero() {-self.z} else {self.z},
    }
  }
}

impl<T: VecTraits<T>> Add<Vec3<T>> for Vec3<T> {
  type Output = Vec3<T>;
  fn add(self, other: Self) -> Self {
    Self {
      x: self.x + other.x,
      y: self.y + other.y,
      z: self.z + other.z,
    }
  }
}
impl<T: VecTraits<T>> AddAssign<Vec3<T>> for Vec3<T> {
  fn add_assign(&mut self, other: Self) {
    self.x += other.x;
    self.y += other.y;
    self.z += other.z;
  }
}

impl<T: VecTraits<T>> Sub<Vec3<T>> for Vec3<T> {
  type Output = Vec3<T>;
  fn sub(self, other: Self) -> Self {
    Self {
      x: self.x - other.x,
      y: self.y - other.y,
      z: self.z - other.z,
    }
  }
}
impl<T: VecTraits<T>> SubAssign<Vec3<T>> for Vec3<T> {
  fn sub_assign(&mut self, other: Self) {
    self.x -= other.x;
    self.y -= other.y;
    self.z -= other.z;
  }
}

impl<T: VecTraits<T>> Mul<Vec3<T>> for Vec3<T> {
  type Output = Vec3<T>;
  fn mul(self, other: Self) -> Self {
    Self {
      x: self.x * other.x,
      y: self.y * other.y,
      z: self.z * other.z,
    }
  }
}
impl<T: VecTraits<T>> Mul<T> for Vec3<T> {
  type Output = Vec3<T>;
  fn mul(self, other: T) -> Self {
    Self {
      x: self.x * other,
      y: self.y * other,
      z: self.z * other,
    }
  }
}
impl<T: VecTraits<T>> MulAssign<Vec3<T>> for Vec3<T> {
  fn mul_assign(&mut self, other: Self) {
    self.x *= other.x;
    self.y *= other.y;
    self.z *= other.z;
  }
}

impl<T: VecTraits<T>> Div<Vec3<T>> for Vec3<T> {
  type Output = Vec3<T>;
  fn div(self, other: Self) -> Self {
    Self {
      x: self.x / other.x,
      y: self.y / other.y,
      z: self.z / other.z,
    }
  }
}
impl<T: VecTraits<T>> Div<T> for Vec3<T> {
  type Output = Vec3<T>;
  fn div(self, other: T) -> Self {
    Self {
      x: self.x / other,
      y: self.y / other,
      z: self.z / other,
    }
  }
}
impl<T: VecTraits<T>> DivAssign<Vec3<T>> for Vec3<T> {
  fn div_assign(&mut self, other: Self) {
    self.x /= other.x;
    self.y /= other.y;
    self.z /= other.z;
  }
}

impl<T: VecTraits<T> + PowAble<T, Output = T>> Vec3<T> {
  pub fn pow(self, other: Self) -> Self {
    Self {
      x: self.x.pow(other.x),
      y: self.y.pow(other.y),
      z: self.z.pow(other.z),
    }
  }
}

// Vector Ops
impl<T: PartialOrd + VecTraits<T>> Vec3<T> {
  pub fn min(a: Self, b: Self) -> Self {
    Self {
      x: if a.x < b.x { a.x } else { b.x },
      y: if a.y < b.y { a.y } else { b.y },
      z: if a.z < b.z { a.z } else { b.z },
    }
  }

  pub fn max(a: Self, b: Self) -> Self {
    Self {
      x: if a.x > b.x { a.x } else { b.x },
      y: if a.y > b.y { a.y } else { b.y },
      z: if a.z > b.z { a.z } else { b.z },
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

    if v.z < lo.z {
      v.z = lo.z;
    }
    if v.z > hi.z {
      v.z = hi.z;
    }
    v
  }
}

impl<T: VecTraits<T>> Vec3<T> {
  pub fn dot(a: Self, b: Self) -> T {
    a.x * b.x + a.y * b.y + a.z * b.z
  }
}

impl<T: SqrtAble<Output = T> + VecTraits<T>> Vec3<T> {
  pub fn len(v: Self) -> T {
    Self::dot(v, v).sqrt()
  }
}

impl<T: SqrtAble<Output = T> + VecTraits<T>> Vec3<T> {
  pub fn normalize(v: Self) -> Self {
    v / Self::len(v)
  }
}

impl<T: SqrtAble<Output = T> + VecTraits<T>> Vec3<T> {
  pub fn lerp(a: Self, b: Self, t: T) -> Self {
    a + (b - a) * t
  }
}

impl<T: VecTraits<T>> Vec3<T> {
  pub fn cross(a: Self, b: Self) -> Self {
    Self {
      x: a.y * b.z - a.z * b.y,
      y: a.z * b.x - a.x * b.z,
      z: a.x * b.y - a.y * b.x,
    }
  }
}
impl<T: PartialEq + VecTraits<T> + Identity<Output = T>> Vec3<T> {
  pub fn perpendicular(a: Self) -> Self {
    let x = Vec3::cross(a, Self::new(T::one(), T::zero(), T::zero()));
    if x == Self::zero() {
      Self::new(T::zero(), T::one(), T::zero())
    } else {
      x
    }
  }
}

impl<T: std::fmt::Display> std::fmt::Display for Vec3<T> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{{{}, {}, {}}}", self.x, self.y, self.z)
  }
}

pub type Vec3f = Vec3<f32>;
pub type Vec3d = Vec3<f64>;
pub type Vec3i = Vec3<i32>;
pub type Vec3u = Vec3<u32>;
pub type Vec3s = Vec3<usize>;

#[macro_export]
macro_rules! vec3 {
  () => {
    $crate::Vec3::default()
  };
  ($x:expr) => {
    $crate::Vec3::new($x, $x, $x)
  };
  ($x:expr, ..) => {{
    let mut v = $crate::Vec3::default();
    v.x = $x;
    v
  }};
  ($x:expr, $y:expr, ..) => {{
    let mut v = $crate::Vec3::default();
    v.x = $x;
    v.y = $y;
    v
  }};
  ($x:expr, $y:expr, $z:expr) => {
    $crate::Vec3::new($x, $y, $z)
  };
}

#[cfg(test)]
mod tests {
  use crate::vec3::*;

  #[test]
  fn ctor() {
    let d = Vec3f::default();
    let r = Vec3f::repeat(1.2);
    let n = Vec3::new(0.5, 1.3, 2.7);
    assert_eq!(d.x, 0.0);
    assert_eq!(d.y, 0.0);
    assert_eq!(d.z, 0.0);
    assert_eq!(r.x, 1.2);
    assert_eq!(r.y, 1.2);
    assert_eq!(r.z, 1.2);
    assert_eq!(n.x, 0.5);
    assert_eq!(n.y, 1.3);
    assert_eq!(n.z, 2.7);
  }

  #[test]
  fn ctor_macro() {
    let a = vec3!(1.2, ..);
    assert_eq!(a.x, 1.2);
    assert_eq!(a.y, 0.0);
    assert_eq!(a.z, 0.0);
    let a = vec3!(1, 2, ..);
    assert_eq!(a.x, 1);
    assert_eq!(a.y, 2);
    assert_eq!(a.z, 0);
    let a = vec3!(1.2);
    assert_eq!(a.x, 1.2);
    assert_eq!(a.y, 1.2);
    assert_eq!(a.z, 1.2);
    let a = vec3!(0.5, 1.3, 2.1);
    assert_eq!(a.x, 0.5);
    assert_eq!(a.y, 1.3);
    assert_eq!(a.z, 2.1);
  }

  #[test]
  fn eq() {
    let a = vec3!(1.2);
    let b = vec3!(0.5, 1.3, ..);
    let c = vec3!(0.5, 1.3, 2.1);
    let d = vec3!(0.5, 1.3, 2.1);
    assert_eq!(a == b, false);
    assert_eq!(a != b, true);
    assert_eq!(b == c, false);
    assert_eq!(c == d, true);
  }

  #[test]
  fn add() {
    let a = vec3!(1, 2, 3);
    let b = vec3!(5, 1, 3);
    let mut c = a + b;
    assert_eq!(c, vec3!(6, 3, 6));

    c += b;
    assert_eq!(c, vec3!(11, 4, 9));
  }

  #[test]
  fn sub() {
    let a = vec3!(1, 2, 3);
    let b = vec3!(5, 1, 3);
    let mut c = b - a;
    assert_eq!(c, vec3!(4, -1, 0));

    c -= b;
    assert_eq!(c, vec3!(-1, -2, -3));
  }

  #[test]
  fn mul() {
    let a = vec3!(2, 3, 4);
    let b = vec3!(5, 1, 3);
    let mut c = a * b;
    assert_eq!(c, vec3!(10, 3, 12));

    c *= a;
    assert_eq!(c, vec3!(20, 9, 48));

    c = c * 2;
    assert_eq!(c, vec3!(40, 18, 96));
  }

  #[test]
  fn div() {
    let a = vec3!(50, 30, 100);
    let b = vec3!(2, 3, 4);
    let mut c = a / b;
    assert_eq!(c, vec3!(25, 10, 25));

    c /= b;
    assert_eq!(c, vec3!(12, 3, 6));

    c = c / 2;
    assert_eq!(c, vec3!(6, 1, 3));
  }

  #[test]
  fn minmax() {
    let a = vec3!(4);
    let b = vec3!(5, 3, 1);
    let c = Vec3::min(a, b);
    assert_eq!(c, vec3!(4, 3, 1));
    let c = Vec3::max(a, b);
    assert_eq!(c, vec3!(5, 4, 4));
  }

  #[test]
  fn len() {
    assert_eq!(Vec3f::len(vec3!(2, 0, 0).into()), 2.0);
    assert_eq!(Vec3::len(vec3!(2.0, 1.0, 1.0)), f32::sqrt(6.0));
    assert_eq!(Vec3::len(vec3!(2.0, 2.0, 3.0)), f32::sqrt(17.0));
    assert!(f32::abs(Vec3::len(Vec3::normalize(vec3!(2.0, 2.0, 2.0))) - 1.0) < 0.0001);
  }

  #[test]
  fn cast() {
    let i = vec3!(1, 2, 3);
    let f = vec3!(1.1, 2.2, 3.3);

    let ii = i * f.into();
    let fi = f * i.into();

    assert_eq!(ii, vec3!(1, 4, 9));
    assert_eq!(fi, vec3!(1.1, 4.4, 3.0 * 3.3));
  }
}
