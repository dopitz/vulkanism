use crate::common_traits::*;
use crate::Vec3;

#[repr(C)]
#[derive(Default, Clone, Copy, Debug)]
pub struct Mat3<T> {
  pub col: [Vec3<T>; 3],
}

// Constructors
impl<T: MatTraits<T>> Mat3<T> {
  pub fn repeat(v: T) -> Self {
    let c = Vec3::repeat(v);
    Self { col: [c, c, c] }
  }

  pub fn new() -> Self {
    Default::default()
  }

  pub fn identity() -> Self {
    let mut m = Self::new();
    m.col[0].x = T::one();
    m.col[1].y = T::one();
    m.col[2].z = T::one();
    m
  }

  pub fn from_slice_rowmajor(m: &[T]) -> Self {
    Self {
      col: [
        Vec3::new(m[0], m[3], m[6]),
        Vec3::new(m[1], m[4], m[7]),
        Vec3::new(m[2], m[5], m[8]),
      ],
    }
  }

  pub fn from_slice_colmajor(m: &[T]) -> Self {
    Self {
      col: [
        Vec3::new(m[0], m[1], m[2]),
        Vec3::new(m[3], m[4], m[5]),
        Vec3::new(m[6], m[7], m[8]),
      ],
    }
  }

  pub fn from_rows(r0: Vec3<T>, r1: Vec3<T>, r2: Vec3<T>) -> Self {
    Self::from_cols(r0, r1, r2).transpose()
  }

  pub fn from_cols(c0: Vec3<T>, c1: Vec3<T>, c2: Vec3<T>) -> Self {
    Self { col: [c0, c1, c2] }
  }

  pub fn from<U: Copy + VecInto<T>>(m: Mat3<U>) -> Self {
    Self {
      col: [m.col[0].into(), m.col[1].into(), m.col[2].into()],
    }
  }

  pub fn into<U>(self) -> Mat3<U>
  where
    T: VecInto<U>,
  {
    Mat3::<U> {
      col: [self.col[0].into(), self.col[1].into(), self.col[2].into()],
    }
  }
}

// Compare
impl<T: PartialEq> PartialEq for Mat3<T> {
  fn eq(&self, other: &Self) -> bool {
    for (a, b) in self.col.iter().zip(other.col.iter()) {
      if a != b {
        return false;
      }
    }
    true
  }
}
impl<T: Eq> Eq for Mat3<T> {}

// Arithmetic Ops
impl<T: MatTraits<T>> Neg for Mat3<T> {
  type Output = Mat3<T>;
  fn neg(self) -> Self {
    let mut m = self;
    for a in m.col.iter_mut() {
      *a = -*a;
    }
    m
  }
}

impl<T: MatTraits<T>> Add<Mat3<T>> for Mat3<T> {
  type Output = Mat3<T>;
  fn add(self, mut other: Self) -> Self {
    for (a, b) in self.col.iter().zip(other.col.iter_mut()) {
      *b = *a + *b;
    }
    other
  }
}
impl<T: MatTraits<T>> AddAssign<Mat3<T>> for Mat3<T> {
  fn add_assign(&mut self, other: Self) {
    for (a, b) in self.col.iter_mut().zip(other.col.iter()) {
      *a = *a + *b;
    }
  }
}

impl<T: MatTraits<T>> Sub<Mat3<T>> for Mat3<T> {
  type Output = Mat3<T>;
  fn sub(self, mut other: Self) -> Self {
    for (a, b) in self.col.iter().zip(other.col.iter_mut()) {
      *b = *a - *b;
    }
    other
  }
}
impl<T: MatTraits<T>> SubAssign<Mat3<T>> for Mat3<T> {
  fn sub_assign(&mut self, other: Self) {
    for (a, b) in self.col.iter_mut().zip(other.col.iter()) {
      *a = *a - *b;
    }
  }
}

impl<T: MatTraits<T>> Mul<Mat3<T>> for Mat3<T> {
  type Output = Mat3<T>;
  fn mul(self, other: Self) -> Self {
    let mut res = Self::repeat(T::zero());
    for r in 0..3 {
      for c in 0..3 {
        let v = &mut res.col[c][r];
        for s in 0..3 {
          *v += self.at(r, s) * other.at(s, c);
        }
      }
    }
    res
  }
}
impl<T: MatTraits<T> + VecTraits<T>> Mul<Vec3<T>> for Mat3<T> {
  type Output = Vec3<T>;
  fn mul(self, v: Vec3<T>) -> Vec3<T> {
    let mut res = Vec3::repeat(T::zero());
    for r in 0..3 {
      for c in 0..3 {
        res[r] += self.at(r, c) * v[c];
      }
    }
    res
  }
}
impl<T: MatTraits<T>> Mul<T> for Mat3<T> {
  type Output = Mat3<T>;
  fn mul(self, other: T) -> Self {
    let mut m = self;
    for a in m.col.iter_mut() {
      *a = *a * other;
    }
    m
  }
}
impl<T: MatTraits<T>> MulAssign<Mat3<T>> for Mat3<T> {
  fn mul_assign(&mut self, other: Self) {
    *self = *self * other;
  }
}

impl<T: MatTraits<T>> Div<T> for Mat3<T> {
  type Output = Mat3<T>;
  fn div(self, other: T) -> Self {
    let mut m = self;
    for a in m.col.iter_mut() {
      *a = *a / other;
    }
    m
  }
}

// Element access
impl<T: MatTraits<T>> Mat3<T> {
  pub fn at(&self, row: usize, col: usize) -> T {
    self.col[col][row]
  }

  pub fn row(&self, i: usize) -> Vec3<T> {
    Vec3::<T> {
      x: self.col[0][i],
      y: self.col[1][i],
      z: self.col[2][i],
    }
  }

  pub fn col(&self, i: usize) -> Vec3<T> {
    self.col[i]
  }

  pub fn set(&mut self, row: usize, col: usize, value: T) {
    self.col[col][row] = value;
  }

  pub fn set_row(&mut self, i: usize, row: Vec3<T>) {
    self.col[0][i] = row.x;
    self.col[1][i] = row.y;
    self.col[2][i] = row.z;
  }

  pub fn set_col(&mut self, i: usize, col: Vec3<T>) {
    self.col[i] = col;
  }
}

// Opertators
impl<T: MatTraits<T>> Mat3<T>
where
  Mat3<T>: Div<T, Output = Mat3<T>>,
{
  pub fn det(&self) -> T {
    Vec3::dot(self.row(0), self.adjugate().col[0])
  }

  pub fn adjugate(&self) -> Self {
    Self {
      col: [
        Vec3::new(
          self.col[1].y * self.col[2].z - self.col[2].y * self.col[1].z,
          -(self.col[0].y * self.col[2].z - self.col[2].y * self.col[0].z),
          self.col[0].y * self.col[1].z - self.col[1].y * self.col[0].z,
        ),
        Vec3::new(
          -(self.col[1].x * self.col[2].z - self.col[2].x * self.col[1].z),
          self.col[0].x * self.col[2].z - self.col[2].x * self.col[0].z,
          -(self.col[0].x * self.col[1].z - self.col[1].x * self.col[0].z),
        ),
        Vec3::new(
          self.col[1].x * self.col[2].y - self.col[2].x * self.col[1].y,
          -(self.col[0].x * self.col[2].y - self.col[2].x * self.col[0].y),
          self.col[0].x * self.col[1].y - self.col[1].x * self.col[0].y,
        ),
      ],
    }
  }

  pub fn inverse(&self) -> Self {
    let adj = self.adjugate();
    let det = Vec3::dot(self.row(0), adj.col[0]);
    adj * (T::one() / det)
  }

  pub fn transpose(&self) -> Self {
    Self {
      col: [
        Vec3::new(self.col[0].x, self.col[1].x, self.col[2].x),
        Vec3::new(self.col[0].y, self.col[1].y, self.col[2].y),
        Vec3::new(self.col[0].z, self.col[1].z, self.col[2].z),
      ],
    }
  }
}

// Generators
impl<T: MatTraits<T>> Mat3<T>
where
  Mat3<T>: Mul<Mat3<T>, Output = Mat3<T>>,
{
  pub fn rotation_x(radians: T) -> Self {
    let s = radians.sin();
    let c = radians.cos();

    Self {
      col: [
        Vec3::new(T::one(), T::zero(), T::zero()),
        Vec3::new(T::zero(), c, -s),
        Vec3::new(T::zero(), s, c),
      ],
    }
  }

  pub fn rotation_y(radians: T) -> Self {
    let s = radians.sin();
    let c = radians.cos();

    Self {
      col: [
        Vec3::new(c, T::zero(), s),
        Vec3::new(T::zero(), T::one(), T::zero()),
        Vec3::new(-s, T::zero(), c),
      ],
    }
  }

  pub fn rotation_z(radians: T) -> Self {
    let s = radians.sin();
    let c = radians.cos();

    Self {
      col: [
        Vec3::new(c, -s, T::zero()),
        Vec3::new(s, c, T::zero()),
        Vec3::new(T::zero(), T::zero(), T::one()),
      ],
    }
  }

  pub fn roll_pitch_yaw(roll: T, pitch: T, yaw: T) -> Self {
    Mat3::rotation_z(roll) * Mat3::rotation_x(pitch) * Mat3::rotation_y(yaw)
  }

  pub fn rotation_zyx(z: T, y: T, x: T) -> Self {
    Mat3::rotation_z(z) * Mat3::rotation_y(y) * Mat3::rotation_x(x)
  }

  pub fn rotation_axis(axis: Vec3<T>, angle: T) -> Self {
    let s = angle.sin();
    let c = angle.cos();
    let t = T::one() - c;

    Self {
      col: [
        Vec3::new(
          t * axis.x * axis.x + c,
          t * axis.x * axis.y - s * axis.z,
          t * axis.x * axis.z + s * axis.y,
        ),
        Vec3::new(
          t * axis.x * axis.y + s * axis.z,
          t * axis.y * axis.y + c,
          t * axis.y * axis.z - s * axis.x,
        ),
        Vec3::new(
          t * axis.x * axis.z - s * axis.y,
          t * axis.y * axis.z + s * axis.x,
          t * axis.z * axis.z + c,
        ),
      ],
    }
  }
}

pub type Mat3f = Mat3<f32>;
pub type Mat3d = Mat3<f64>;

#[cfg(test)]
mod tests {
  use crate::mat3::*;

  #[test]
  fn ctor() {
    let d = Mat3f::default();
    let r = Mat3f::repeat(1.2);
    let c = Vec3::repeat(0.0);
    for i in d.col.iter() {
      assert_eq!(*i, c);
    }
    let c = Vec3::repeat(1.2);
    for i in r.col.iter() {
      assert_eq!(*i, c);
    }
  }

  #[test]
  fn ident() {
    let r = Mat3::<f32>::identity();
    assert_eq!(r.col[0], Vec3::new(1.0, 0.0, 0.0));
    assert_eq!(r.col[1], Vec3::new(0.0, 1.0, 0.0));
    assert_eq!(r.col[2], Vec3::new(0.0, 0.0, 1.0));
    let r = Mat3f::identity();
    assert_eq!(r.col[0], Vec3::new(1.0, 0.0, 0.0));
    assert_eq!(r.col[1], Vec3::new(0.0, 1.0, 0.0));
    assert_eq!(r.col[2], Vec3::new(0.0, 0.0, 1.0));
  }

  #[test]
  fn transpose() {
    let mut i = Mat3f::identity();
    i.col[1].x = 1.0;
    let t = i.clone().transpose();
    assert_eq!(t.col[0].y, 1.0);
    assert_eq!(i, t.transpose());
  }

  #[test]
  fn rowcol() {
    let m = Mat3f::from_slice_rowmajor(&[0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0]);
    assert_eq!(m.row(0), vec3!(0.0, 1.0, 2.0));
    assert_eq!(m.row(1), vec3!(3.0, 4.0, 5.0));
    assert_eq!(m.row(2), vec3!(6.0, 7.0, 8.0));
    assert_eq!(m.col(0), vec3!(0.0, 3.0, 6.0));
    assert_eq!(m.col(1), vec3!(1.0, 4.0, 7.0));
    assert_eq!(m.col(2), vec3!(2.0, 5.0, 8.0));
    assert_eq!(m, Mat3f::from_slice_colmajor(&[0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0]).transpose())
  }

  #[test]
  fn det() {
    let m = Mat3f::from_slice_rowmajor(&[0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0]);
    assert_eq!(m.det(), 0.0);
  }
}
