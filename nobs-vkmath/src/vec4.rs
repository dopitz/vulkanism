use crate::common_traits::*;

#[repr(C)]
#[derive(Default, Clone, Copy, Debug)]
pub struct Vec4<T> {
  pub x: T,
  pub y: T,
  pub z: T,
  pub w: T,
}

// Constructors
impl<T: Copy> Vec4<T> {
  pub fn repeat(v: T) -> Self
  where
    T: Copy,
  {
    Self { x: v, y: v, z: v, w: v }
  }

  pub fn new(x: T, y: T, z: T, w: T) -> Self {
    Self { x, y, z, w }
  }

  pub fn from<U: VecInto<T>>(v: Vec4<U>) -> Self {
    Self {
      x: v.x.vec_into(),
      y: v.y.vec_into(),
      z: v.z.vec_into(),
      w: v.w.vec_into(),
    }
  }

  pub fn into<U>(self) -> Vec4<U>
  where
    T: VecInto<U>,
  {
    Vec4::<U> {
      x: self.x.vec_into(),
      y: self.y.vec_into(),
      z: self.z.vec_into(),
      w: self.w.vec_into(),
    }
  }
}

// Compare
impl<T: PartialEq> PartialEq for Vec4<T> {
  fn eq(&self, other: &Self) -> bool {
    self.x == other.x && self.y == other.y && self.z == other.z && self.w == other.w
  }
}
impl<T: Eq> Eq for Vec4<T> {}

// Element Access
impl<T> Index<usize> for Vec4<T> {
  type Output = T;

  fn index(&self, i: usize) -> &T {
    match i {
      0 => &self.x,
      1 => &self.y,
      2 => &self.z,
      3 => &self.w,
      n => panic!("vec4 used with index {}", n),
    }
  }
}
impl<T> IndexMut<usize> for Vec4<T> {
  fn index_mut(&mut self, i: usize) -> &mut T {
    match i {
      0 => &mut self.x,
      1 => &mut self.y,
      2 => &mut self.z,
      3 => &mut self.w,
      n => panic!("vec4 used with index {}", n),
    }
  }
}

// Arithmetic Ops
impl<T: Neg<Output = T>> Neg for Vec4<T> {
  type Output = Vec4<T>;
  fn neg(self) -> Self {
    Self {
      x: -self.x,
      y: -self.y,
      z: -self.z,
      w: -self.w,
    }
  }
}

impl<T: VecTraits<T>> Add<Vec4<T>> for Vec4<T> {
  type Output = Vec4<T>;
  fn add(self, other: Self) -> Self {
    Self {
      x: self.x + other.x,
      y: self.y + other.y,
      z: self.z + other.z,
      w: self.w + other.w,
    }
  }
}
impl<T: VecTraits<T>> AddAssign<Vec4<T>> for Vec4<T> {
  fn add_assign(&mut self, other: Self) {
    self.x += other.x;
    self.y += other.y;
    self.z += other.z;
    self.w += other.w;
  }
}

impl<T: VecTraits<T>> Sub<Vec4<T>> for Vec4<T> {
  type Output = Vec4<T>;
  fn sub(self, other: Self) -> Self {
    Self {
      x: self.x - other.x,
      y: self.y - other.y,
      z: self.z - other.z,
      w: self.w - other.w,
    }
  }
}
impl<T: VecTraits<T>> SubAssign<Vec4<T>> for Vec4<T> {
  fn sub_assign(&mut self, other: Self) {
    self.x -= other.x;
    self.y -= other.y;
    self.z -= other.z;
    self.w -= other.w;
  }
}

impl<T: VecTraits<T>> Mul<Vec4<T>> for Vec4<T> {
  type Output = Vec4<T>;
  fn mul(self, other: Self) -> Self {
    Self {
      x: self.x * other.x,
      y: self.y * other.y,
      z: self.z * other.z,
      w: self.w * other.w,
    }
  }
}
impl<T: VecTraits<T>> Mul<T> for Vec4<T> {
  type Output = Vec4<T>;
  fn mul(self, other: T) -> Self {
    Self {
      x: self.x * other,
      y: self.y * other,
      z: self.z * other,
      w: self.w * other,
    }
  }
}
impl<T: VecTraits<T>> MulAssign<Vec4<T>> for Vec4<T> {
  fn mul_assign(&mut self, other: Self) {
    self.x *= other.x;
    self.y *= other.y;
    self.z *= other.z;
    self.w *= other.w;
  }
}

impl<T: VecTraits<T>> Div<Vec4<T>> for Vec4<T> {
  type Output = Vec4<T>;
  fn div(self, other: Self) -> Self {
    Self {
      x: self.x / other.x,
      y: self.y / other.y,
      z: self.z / other.z,
      w: self.w / other.w,
    }
  }
}
impl<T: VecTraits<T>> Div<T> for Vec4<T> {
  type Output = Vec4<T>;
  fn div(self, other: T) -> Self {
    Self {
      x: self.x / other,
      y: self.y / other,
      z: self.z / other,
      w: self.w / other,
    }
  }
}
impl<T: VecTraits<T>> DivAssign<Vec4<T>> for Vec4<T> {
  fn div_assign(&mut self, other: Self) {
    self.x /= other.x;
    self.y /= other.y;
    self.z /= other.z;
    self.w /= other.w;
  }
}

// Vector Ops
impl<T: Ord + VecTraits<T>> Vec4<T> {
  pub fn min(a: Self, b: Self) -> Self {
    Self {
      x: std::cmp::min(a.x, b.x),
      y: std::cmp::min(a.y, b.y),
      z: std::cmp::min(a.z, b.z),
      w: std::cmp::min(a.w, b.w),
    }
  }

  pub fn max(a: Self, b: Self) -> Self {
    Self {
      x: std::cmp::max(a.x, b.x),
      y: std::cmp::max(a.y, b.y),
      z: std::cmp::max(a.z, b.z),
      w: std::cmp::max(a.w, b.w),
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

    if v.w < lo.w {
      v.w = lo.w;
    }
    if v.w > hi.w {
      v.w = hi.w;
    }
    v
  }
}

impl<T: VecTraits<T>> Vec4<T> {
  pub fn dot(a: Self, b: Self) -> T {
    a.x * b.x + a.y * b.y + a.z * b.z + a.w * b.w
  }
}

impl<T: SqrtAble<Output = T> + VecTraits<T>> Vec4<T> {
  pub fn len(v: Self) -> T {
    Self::dot(v, v).sqrt()
  }
}

impl<T: SqrtAble<Output = T> + VecTraits<T>> Vec4<T> {
  pub fn normalize(v: Self) -> Self {
    v / Self::len(v)
  }
}

impl<T: SqrtAble<Output = T> + VecTraits<T>> Vec4<T> {
  pub fn lerp(a: Self, b: Self, t: T) -> Self {
    a + (b - a) * t
  }
}

pub type Vec4f = Vec4<f32>;
pub type Vec4d = Vec4<f64>;
pub type Vec4i = Vec4<i32>;
pub type Vec4u = Vec4<u32>;
pub type Vec4s = Vec4<usize>;

#[macro_export]
macro_rules! vec4 {
  () => {
    $crate::Vec4::default()
  };
  ($x:expr) => {
    $crate::Vec4::new($x, $x, $x, $x)
  };
  ($x:expr, ..) => {{
    let mut v = $crate::Vec4::default();
    v.x = $x;
    v
  }};
  ($x:expr, $y:expr, ..) => {{
    let mut v = $crate::Vec4::default();
    v.x = $x;
    v.y = $y;
    v
  }};
  ($x:expr, $y:expr, $z:expr, ..) => {{
    let mut v = $crate::Vec4::default();
    v.x = $x;
    v.y = $y;
    v.z = $z;
    v
  }};
  ($x:expr, $y:expr, $z:expr, $w:expr) => {
    $crate::Vec4::new($x, $y, $z, $w)
  };
}

#[cfg(test)]
mod tests {
  use crate::vec4::*;

  #[test]
  fn ctor() {
    let d = Vec4f::default();
    let r = Vec4f::repeat(1.2);
    let n = Vec4::new(0.5, 1.3, 2.7, 3.1);
    assert_eq!(d.x, 0.0);
    assert_eq!(d.y, 0.0);
    assert_eq!(d.z, 0.0);
    assert_eq!(d.w, 0.0);
    assert_eq!(r.x, 1.2);
    assert_eq!(r.y, 1.2);
    assert_eq!(r.z, 1.2);
    assert_eq!(r.w, 1.2);
    assert_eq!(n.x, 0.5);
    assert_eq!(n.y, 1.3);
    assert_eq!(n.z, 2.7);
    assert_eq!(n.w, 3.1);
  }

  #[test]
  fn ctor_macro() {
    let a = vec4!(1.2, ..);
    assert_eq!(a.x, 1.2);
    assert_eq!(a.y, 0.0);
    assert_eq!(a.z, 0.0);
    assert_eq!(a.w, 0.0);
    let a = vec4!(1, 2, ..);
    assert_eq!(a.x, 1);
    assert_eq!(a.y, 2);
    assert_eq!(a.z, 0);
    assert_eq!(a.w, 0);
    let a = vec4!(1.2);
    assert_eq!(a.x, 1.2);
    assert_eq!(a.y, 1.2);
    assert_eq!(a.z, 1.2);
    assert_eq!(a.w, 1.2);
    let a = vec4!(0.5, 1.3, 2.1, 3.1);
    assert_eq!(a.x, 0.5);
    assert_eq!(a.y, 1.3);
    assert_eq!(a.z, 2.1);
    assert_eq!(a.w, 3.1);
  }

  #[test]
  fn eq() {
    let a = vec4!(1.2);
    let b = vec4!(0.5, 1.3, ..);
    let c = vec4!(0.5, 1.3, 2.1, 3.2);
    let d = vec4!(0.5, 1.3, 2.1, 3.2);
    assert_eq!(a == b, false);
    assert_eq!(a != b, true);
    assert_eq!(b == c, false);
    assert_eq!(c == d, true);
  }

  #[test]
  fn add() {
    let a = vec4!(1, 2, 3, 4);
    let b = vec4!(5, 1, 3, 3);
    let mut c = a + b;
    assert_eq!(c, vec4!(6, 3, 6, 7));

    c += b;
    assert_eq!(c, vec4!(11, 4, 9, 10));
  }

  #[test]
  fn sub() {
    let a = vec4!(1, 2, 3, 4);
    let b = vec4!(5, 1, 3, 3);
    let mut c = b - a;
    assert_eq!(c, vec4!(4, -1, 0, -1));

    c -= b;
    assert_eq!(c, vec4!(-1, -2, -3, -4));
  }

  #[test]
  fn mul() {
    let a = vec4!(2, 3, 4, 5);
    let b = vec4!(5, 1, 3, 3);
    let mut c = a * b;
    assert_eq!(c, vec4!(10, 3, 12, 15));

    c *= a;
    assert_eq!(c, vec4!(20, 9, 48, 75));

    c = c * 2;
    assert_eq!(c, vec4!(40, 18, 96, 150));
  }

  #[test]
  fn div() {
    let a = vec4!(50, 30, 100, 60);
    let b = vec4!(2, 3, 4, 5);
    let mut c = a / b;
    assert_eq!(c, vec4!(25, 10, 25, 12));

    c /= b;
    assert_eq!(c, vec4!(12, 3, 6, 2));

    c = c / 2;
    assert_eq!(c, vec4!(6, 1, 3, 1));
  }

  #[test]
  fn minmax() {
    let a = vec4!(4);
    let b = vec4!(5, 3, 1, 10);
    let c = Vec4::min(a, b);
    assert_eq!(c, vec4!(4, 3, 1, 4));
    let c = Vec4::max(a, b);
    assert_eq!(c, vec4!(5, 4, 4, 10));
  }

  #[test]
  fn len() {
    assert_eq!(Vec4f::len(vec4!(2, 0, 0, 0).into()), 2.0);
    assert_eq!(Vec4::len(vec4!(2.0, 1.0, 1.0, 3.0)), f32::sqrt(15.0));
    assert_eq!(Vec4::len(vec4!(2.0, 2.0, 3.0, 4.0)), f32::sqrt(33.0));
    assert!(f32::abs(Vec4::len(Vec4::normalize(vec4!(2.0, 2.0, 2.0, 2.0))) - 1.0) < 0.0001);
  }

  #[test]
  fn cast() {
    let i = vec4!(1, 2, 3, 4);
    let f = vec4!(1.1, 2.2, 3.3, 4.4);

    let ii = i * f.into();
    let fi = f * i.into();

    assert_eq!(ii, vec4!(1, 4, 9, 16));
    assert_eq!(fi, vec4!(1.1, 4.4, 3.0 * 3.3, 4.0 * 4.4));
  }
}

