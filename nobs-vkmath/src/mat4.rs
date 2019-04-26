use crate::common_traits::*;
use crate::Mat3;
use crate::Vec3;
use crate::Vec4;

#[repr(C)]
#[derive(Default, Clone, Copy, Debug)]
pub struct Mat4<T> {
  pub col: [Vec4<T>; 4],
}

// Constructors
impl<T: MatTraits<T>> Mat4<T> {
  pub fn repeat(v: T) -> Self {
    let c = Vec4::repeat(v);
    Self { col: [c, c, c, c] }
  }

  pub fn new() -> Self {
    Default::default()
  }

  pub fn identity() -> Self {
    let mut m = Self::new();
    m.col[0].x = T::one();
    m.col[1].y = T::one();
    m.col[2].z = T::one();
    m.col[3].w = T::one();
    m
  }

  pub fn from_slice_rowmajor(m: &[T]) -> Self {
    Self {
      col: [
        Vec4::new(m[0], m[4], m[8], m[12]),
        Vec4::new(m[1], m[5], m[9], m[13]),
        Vec4::new(m[2], m[6], m[10], m[14]),
        Vec4::new(m[3], m[7], m[11], m[15]),
      ],
    }
  }

  pub fn from_slice_colmajor(m: &[T]) -> Self {
    Self {
      col: [
        Vec4::new(m[0], m[1], m[2], m[3]),
        Vec4::new(m[4], m[5], m[6], m[7]),
        Vec4::new(m[8], m[9], m[10], m[11]),
        Vec4::new(m[12], m[13], m[14], m[15]),
      ],
    }
  }

  pub fn from_rows(r0: Vec4<T>, r1: Vec4<T>, r2: Vec4<T>, r3: Vec4<T>) -> Self {
    Self::from_cols(r0, r1, r2, r3).transpose()
  }

  pub fn from_cols(c0: Vec4<T>, c1: Vec4<T>, c2: Vec4<T>, c3: Vec4<T>) -> Self {
    Self { col: [c0, c1, c2, c3] }
  }

  pub fn from<U: Copy + VecInto<T>>(m: Mat4<U>) -> Self {
    Self {
      col: [m.col[0].into(), m.col[1].into(), m.col[2].into(), m.col[3].into()],
    }
  }

  pub fn into<U>(self) -> Mat4<U>
  where
    T: VecInto<U>,
  {
    Mat4::<U> {
      col: [self.col[0].into(), self.col[1].into(), self.col[2].into(), self.col[3].into()],
    }
  }
}

// Compare
impl<T: PartialEq> PartialEq for Mat4<T> {
  fn eq(&self, other: &Self) -> bool {
    for (a, b) in self.col.iter().zip(other.col.iter()) {
      if a != b {
        return false;
      }
    }
    true
  }
}

// Arithmetic Ops
impl<T: MatTraits<T>> Neg for Mat4<T> {
  type Output = Mat4<T>;
  fn neg(self) -> Self {
    let mut m = self;
    for a in m.col.iter_mut() {
      *a = -*a;
    }
    m
  }
}

impl<T: MatTraits<T>> Add<Mat4<T>> for Mat4<T> {
  type Output = Mat4<T>;
  fn add(self, mut other: Self) -> Self {
    for (a, b) in self.col.iter().zip(other.col.iter_mut()) {
      *b = *a + *b;
    }
    other
  }
}
impl<T: MatTraits<T>> AddAssign<Mat4<T>> for Mat4<T> {
  fn add_assign(&mut self, other: Self) {
    for (a, b) in self.col.iter_mut().zip(other.col.iter()) {
      *a += *b;
    }
  }
}

impl<T: MatTraits<T>> Sub<Mat4<T>> for Mat4<T> {
  type Output = Mat4<T>;
  fn sub(self, mut other: Self) -> Self {
    for (a, b) in self.col.iter().zip(other.col.iter_mut()) {
      *b = *a - *b;
    }
    other
  }
}
impl<T: MatTraits<T>> SubAssign<Mat4<T>> for Mat4<T> {
  fn sub_assign(&mut self, other: Self) {
    for (a, b) in self.col.iter_mut().zip(other.col.iter()) {
      *a -= *b;
    }
  }
}

impl<T: MatTraits<T>> Mul<Mat4<T>> for Mat4<T> {
  type Output = Mat4<T>;
  fn mul(self, other: Self) -> Self {
    let mut res = Self::repeat(T::zero());
    for r in 0..4 {
      for c in 0..4 {
        let v = &mut res.col[c][r];
        for s in 0..4 {
          *v += self.at(r, s) * other.at(s, c);
        }
      }
    }
    res
  }
}
impl<T: MatTraits<T> + VecTraits<T>> Mul<Vec4<T>> for Mat4<T> {
  type Output = Vec4<T>;
  fn mul(self, v: Vec4<T>) -> Vec4<T> {
    let mut res = Vec4::repeat(T::zero());
    for r in 0..4 {
      for c in 0..4 {
        res[r] += self.at(r, c) * v[c];
      }
    }
    res
  }
}
impl<T: MatTraits<T>> Mul<T> for Mat4<T> {
  type Output = Mat4<T>;
  fn mul(self, other: T) -> Self {
    let mut m = self;
    for a in m.col.iter_mut() {
      *a = *a * other;
    }
    m
  }
}
impl<T: MatTraits<T>> MulAssign<Mat4<T>> for Mat4<T> {
  fn mul_assign(&mut self, other: Self) {
    *self = *self * other;
  }
}

impl<T: MatTraits<T>> Div<T> for Mat4<T> {
  type Output = Mat4<T>;
  fn div(self, other: T) -> Self {
    let mut m = self;
    for a in m.col.iter_mut() {
      *a = *a / other;
    }
    m
  }
}

// Element access
impl<T: MatTraits<T>> Mat4<T> {
  pub fn at(&self, row: usize, col: usize) -> T {
    self.col[col][row]
  }

  pub fn row(&self, i: usize) -> Vec4<T> {
    Vec4::<T> {
      x: self.col[0][i],
      y: self.col[1][i],
      z: self.col[2][i],
      w: self.col[3][i],
    }
  }

  pub fn col(&self, i: usize) -> Vec4<T> {
    self.col[i]
  }

  pub fn rotation(&self) -> Mat3<T> {
    Mat3::<T> {
      col: [
        Vec3::new(self.col[0].x, self.col[0].y, self.col[0].z),
        Vec3::new(self.col[1].x, self.col[1].y, self.col[1].z),
        Vec3::new(self.col[2].x, self.col[2].y, self.col[2].z),
      ],
    }
  }

  pub fn translation(&self) -> Vec3<T> {
    Vec3::<T> {
      x: self.col[3].x,
      y: self.col[3].y,
      z: self.col[3].z,
    }
  }

  pub fn set(&mut self, row: usize, col: usize, value: T) {
    self.col[col][row] = value;
  }

  pub fn set_row(&mut self, i: usize, row: Vec4<T>) {
    self.col[0][i] = row.x;
    self.col[1][i] = row.y;
    self.col[2][i] = row.z;
    self.col[3][i] = row.w;
  }

  pub fn set_col(&mut self, i: usize, col: Vec4<T>) {
    self.col[i] = col;
  }

  pub fn set_rotation(&mut self, rot: Mat3<T>) {
    self.col[0].x = rot.col[0].x;
    self.col[1].x = rot.col[1].x;
    self.col[2].x = rot.col[2].x;

    self.col[0].y = rot.col[0].y;
    self.col[1].y = rot.col[1].y;
    self.col[2].y = rot.col[2].y;

    self.col[0].z = rot.col[0].z;
    self.col[1].z = rot.col[1].z;
    self.col[2].z = rot.col[2].z;
  }

  pub fn set_translation(&mut self, t: Vec3<T>) {
    self.col[3].x = t.x;
    self.col[3].y = t.y;
    self.col[3].z = t.z;
  }
}

// Opertators
impl<T: MatTraits<T>> Mat4<T>
where
  Mat4<T>: Div<T, Output = Mat4<T>>,
{
  pub fn det(&self) -> T {
    Vec4::dot(self.row(0), self.adjugate().col[0])
  }

  pub fn adjugate(&self) -> Self {
    Self {
      col: [
        Vec4::new(
          self.col[1].y * self.col[2].z * self.col[3].w
            - self.col[1].y * self.col[3].z * self.col[2].w
            - self.col[1].z * self.col[2].y * self.col[3].w
            + self.col[1].z * self.col[3].y * self.col[2].w
            + self.col[1].w * self.col[2].y * self.col[3].z
            - self.col[1].w * self.col[3].y * self.col[2].z,
          -self.col[0].y * self.col[2].z * self.col[3].w
            + self.col[0].y * self.col[3].z * self.col[2].w
            + self.col[0].z * self.col[2].y * self.col[3].w
            - self.col[0].z * self.col[3].y * self.col[2].w
            - self.col[0].w * self.col[2].y * self.col[3].z
            + self.col[0].w * self.col[3].y * self.col[2].z,
          self.col[0].y * self.col[1].z * self.col[3].w
            - self.col[0].y * self.col[3].z * self.col[1].w
            - self.col[0].z * self.col[1].y * self.col[3].w
            + self.col[0].z * self.col[3].y * self.col[1].w
            + self.col[0].w * self.col[1].y * self.col[3].z
            - self.col[0].w * self.col[3].y * self.col[1].z,
          -self.col[0].y * self.col[1].z * self.col[2].w
            + self.col[0].y * self.col[2].z * self.col[1].w
            + self.col[0].z * self.col[1].y * self.col[2].w
            - self.col[0].z * self.col[2].y * self.col[1].w
            - self.col[0].w * self.col[1].y * self.col[2].z
            + self.col[0].w * self.col[2].y * self.col[1].z,
        ),
        Vec4::new(
          -self.col[1].x * self.col[2].z * self.col[3].w
            + self.col[1].x * self.col[3].z * self.col[2].w
            + self.col[1].z * self.col[2].x * self.col[3].w
            - self.col[1].z * self.col[3].x * self.col[2].w
            - self.col[1].w * self.col[2].x * self.col[3].z
            + self.col[1].w * self.col[3].x * self.col[2].z,
          self.col[0].x * self.col[2].z * self.col[3].w
            - self.col[0].x * self.col[3].z * self.col[2].w
            - self.col[0].z * self.col[2].x * self.col[3].w
            + self.col[0].z * self.col[3].x * self.col[2].w
            + self.col[0].w * self.col[2].x * self.col[3].z
            - self.col[0].w * self.col[3].x * self.col[2].z,
          -self.col[0].x * self.col[1].z * self.col[3].w
            + self.col[0].x * self.col[3].z * self.col[1].w
            + self.col[0].z * self.col[1].x * self.col[3].w
            - self.col[0].z * self.col[3].x * self.col[1].w
            - self.col[0].w * self.col[1].x * self.col[3].z
            + self.col[0].w * self.col[3].x * self.col[1].z,
          self.col[0].x * self.col[1].z * self.col[2].w
            - self.col[0].x * self.col[2].z * self.col[1].w
            - self.col[0].z * self.col[1].x * self.col[2].w
            + self.col[0].z * self.col[2].x * self.col[1].w
            + self.col[0].w * self.col[1].x * self.col[2].z
            - self.col[0].w * self.col[2].x * self.col[1].z,
        ),
        Vec4::new(
          self.col[1].x * self.col[2].y * self.col[3].w
            - self.col[1].x * self.col[3].y * self.col[2].w
            - self.col[1].y * self.col[2].x * self.col[3].w
            + self.col[1].y * self.col[3].x * self.col[2].w
            + self.col[1].w * self.col[2].x * self.col[3].y
            - self.col[1].w * self.col[3].x * self.col[2].y,
          -self.col[0].x * self.col[2].y * self.col[3].w
            + self.col[0].x * self.col[3].y * self.col[2].w
            + self.col[0].y * self.col[2].x * self.col[3].w
            - self.col[0].y * self.col[3].x * self.col[2].w
            - self.col[0].w * self.col[2].x * self.col[3].y
            + self.col[0].w * self.col[3].x * self.col[2].y,
          self.col[0].x * self.col[1].y * self.col[3].w
            - self.col[0].x * self.col[3].y * self.col[1].w
            - self.col[0].y * self.col[1].x * self.col[3].w
            + self.col[0].y * self.col[3].x * self.col[1].w
            + self.col[0].w * self.col[1].x * self.col[3].y
            - self.col[0].w * self.col[3].x * self.col[1].y,
          -self.col[0].x * self.col[1].y * self.col[2].w
            + self.col[0].x * self.col[2].y * self.col[1].w
            + self.col[0].y * self.col[1].x * self.col[2].w
            - self.col[0].y * self.col[2].x * self.col[1].w
            - self.col[0].w * self.col[1].x * self.col[2].y
            + self.col[0].w * self.col[2].x * self.col[1].y,
        ),
        Vec4::new(
          -self.col[1].x * self.col[2].y * self.col[3].z
            + self.col[1].x * self.col[3].y * self.col[2].z
            + self.col[1].y * self.col[2].x * self.col[3].z
            - self.col[1].y * self.col[3].x * self.col[2].z
            - self.col[1].z * self.col[2].x * self.col[3].y
            + self.col[1].z * self.col[3].x * self.col[2].y,
          self.col[0].x * self.col[2].y * self.col[3].z
            - self.col[0].x * self.col[3].y * self.col[2].z
            - self.col[0].y * self.col[2].x * self.col[3].z
            + self.col[0].y * self.col[3].x * self.col[2].z
            + self.col[0].z * self.col[2].x * self.col[3].y
            - self.col[0].z * self.col[3].x * self.col[2].y,
          -self.col[0].x * self.col[1].y * self.col[3].z
            + self.col[0].x * self.col[3].y * self.col[1].z
            + self.col[0].y * self.col[1].x * self.col[3].z
            - self.col[0].y * self.col[3].x * self.col[1].z
            - self.col[0].z * self.col[1].x * self.col[3].y
            + self.col[0].z * self.col[3].x * self.col[1].y,
          self.col[0].x * self.col[1].y * self.col[2].z
            - self.col[0].x * self.col[2].y * self.col[1].z
            - self.col[0].y * self.col[1].x * self.col[2].z
            + self.col[0].y * self.col[2].x * self.col[1].z
            + self.col[0].z * self.col[1].x * self.col[2].y
            - self.col[0].z * self.col[2].x * self.col[1].y,
        ),
      ],
    }
  }

  pub fn inverse(&self) -> Self {
    let adj = self.adjugate();
    let det = Vec4::dot(self.row(0), self.adjugate().col[0]);
    adj * (T::one() / det)
  }

  pub fn transpose(&self) -> Self {
    Self {
      col: [
        Vec4::new(self.col[0].x, self.col[1].x, self.col[2].x, self.col[3].x),
        Vec4::new(self.col[0].y, self.col[1].y, self.col[2].y, self.col[3].y),
        Vec4::new(self.col[0].z, self.col[1].z, self.col[2].z, self.col[3].z),
        Vec4::new(self.col[0].w, self.col[1].w, self.col[2].w, self.col[3].w),
      ],
    }
  }
}

// Generators
impl<T: MatTraits<T>> Mat4<T>
where
  i32: VecInto<T>,
  Mat4<T>: Mul<Mat4<T>, Output = Mat4<T>>,
{
  pub fn rotation_x(radians: T) -> Self {
    let mut m = Self::identity();
    m.set_rotation(Mat3::rotation_x(radians));
    m
  }

  pub fn rotation_y(radians: T) -> Self {
    let mut m = Self::identity();
    m.set_rotation(Mat3::rotation_y(radians));
    m
  }

  pub fn rotation_z(radians: T) -> Self {
    let mut m = Self::identity();
    m.set_rotation(Mat3::rotation_z(radians));
    m
  }

  pub fn roll_pitch_yaw(roll: T, pitch: T, yaw: T) -> Self {
    let mut m = Self::identity();
    m.set_rotation(Mat3::roll_pitch_yaw(roll, pitch, yaw));
    m
  }

  pub fn rotation_zyx(z: T, y: T, x: T) -> Self {
    let mut m = Self::identity();
    m.set_rotation(Mat3::rotation_zyx(z, y, x));
    m
  }

  pub fn rotation_axis(axis: Vec3<T>, angle: T) -> Self {
    let mut m = Self::identity();
    m.set_rotation(Mat3::rotation_axis(axis, angle));
    m
  }

  pub fn scale(scale: Vec3<T>) -> Self {
    let mut m = Self::identity();
    m.col[0].x = scale.x;
    m.col[1].y = scale.y;
    m.col[2].z = scale.z;
    m
  }

  pub fn translate(t: Vec3<T>) -> Self {
    let mut m = Self::identity();
    m.set_translation(t);
    m
  }

  pub fn look_at(eye: Vec3<T>, at: Vec3<T>, up: Vec3<T>) -> Self {
    let z = Vec3::normalize(at - eye);
    let x = Vec3::normalize(Vec3::cross(up, z));
    let y = Vec3::normalize(Vec3::cross(z, x));

    Self {
      col: [
        Vec4::new(x.x, y.x, z.x, T::zero()),
        Vec4::new(x.y, y.y, z.y, T::zero()),
        Vec4::new(x.z, y.z, z.z, T::zero()),
        Vec4::new(-Vec3::dot(x, eye), -Vec3::dot(y, eye), -Vec3::dot(z, eye), T::one()),
      ],
    }
  }

  pub fn perspective_lh(fovy: T, aspect: T, zn: T, zf: T) -> Self {
    let tan_y = (fovy / (T::one() + T::one())).tan();
    let y_scale = T::one() / tan_y;
    let x_scale = y_scale / aspect;

    Self {
      col: [
        Vec4::new(x_scale, T::zero(), T::zero(), T::zero()),
        Vec4::new(T::zero(), y_scale, T::zero(), T::zero()),
        Vec4::new(T::zero(), T::zero(), zf / (zf - zn), T::one()),
        Vec4::new(T::zero(), T::zero(), -zn * zf / (zf - zn), T::zero()),
      ],
    }
  }

  pub fn orthographic_lh(width: T, height: T, zn: T, zf: T) -> Self {
    Self {
      col: [
        Vec4::new((T::one() + T::one()) / width, T::zero(), T::zero(), T::zero()),
        Vec4::new(T::zero(), (T::one() + T::one()) / height, T::zero(), T::zero()),
        Vec4::new(T::zero(), T::zero(), T::one() / (zf - zn), T::zero()),
        Vec4::new(T::zero(), T::zero(), T::zero(), zn / (zn - zf)),
      ],
    }
  }
}


pub type Mat4f = Mat4<f32>;
pub type Mat4d = Mat4<f64>;

#[cfg(test)]
mod tests {
  use crate::mat4::*;

  #[test]
  fn ctor() {
    let d = Mat4f::default();
    let r = Mat4f::repeat(1.2);
    let c = Vec4::repeat(0.0);
    for i in d.col.iter() {
      assert_eq!(*i, c);
    }
    let c = Vec4::repeat(1.2);
    for i in r.col.iter() {
      assert_eq!(*i, c);
    }
  }

  #[test]
  fn ident() {
    let r = Mat4::<f32>::identity();
    assert_eq!(r.col[0], Vec4::new(1.0, 0.0, 0.0, 0.0));
    assert_eq!(r.col[1], Vec4::new(0.0, 1.0, 0.0, 0.0));
    assert_eq!(r.col[2], Vec4::new(0.0, 0.0, 1.0, 0.0));
    assert_eq!(r.col[3], Vec4::new(0.0, 0.0, 0.0, 1.0));
    let r = Mat4f::identity();
    assert_eq!(r.col[0], Vec4::new(1.0, 0.0, 0.0, 0.0));
    assert_eq!(r.col[1], Vec4::new(0.0, 1.0, 0.0, 0.0));
    assert_eq!(r.col[2], Vec4::new(0.0, 0.0, 1.0, 0.0));
    assert_eq!(r.col[3], Vec4::new(0.0, 0.0, 0.0, 1.0));
  }

  #[test]
  fn transpose() {
    let mut i = Mat4f::identity();
    i.col[1].x = 1.0;
    let t = i.clone().transpose();
    assert_eq!(t.col[0].y, 1.0);
    assert_eq!(i, t.transpose());
  }

  #[test]
  fn rowcol() {
    let m = Mat4f::from_slice_rowmajor(&[0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 0.0, 1.0, 2.0, 3.0, 4.0, 5.0]);
    assert_eq!(m.row(0), vec4!(0.0, 1.0, 2.0, 3.0));
    assert_eq!(m.row(1), vec4!(4.0, 5.0, 6.0, 7.0));
    assert_eq!(m.row(2), vec4!(8.0, 9.0, 0.0, 1.0));
    assert_eq!(m.row(3), vec4!(2.0, 3.0, 4.0, 5.0));
    assert_eq!(m.col(0), vec4!(0.0, 4.0, 8.0, 2.0));
    assert_eq!(m.col(1), vec4!(1.0, 5.0, 9.0, 3.0));
    assert_eq!(m.col(2), vec4!(2.0, 6.0, 0.0, 4.0));
    assert_eq!(m.col(3), vec4!(3.0, 7.0, 1.0, 5.0));
    assert_eq!(
      m,
      Mat4f::from_slice_colmajor(&[0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 0.0, 1.0, 2.0, 3.0, 4.0, 5.0]).transpose()
    )
  }

  #[test]
  fn det() {
    let m = Mat4f::from_slice_rowmajor(&[0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 0.0, 1.0, 2.0, 3.0, 4.0, 5.0]);
    assert_eq!(m.det(), 0.0);
    let m = Mat4f::from_slice_rowmajor(&[0.0, 1.0, 2.0, 3.0, 4.0, 1.0, 6.0, 7.0, 8.0, 9.0, 1.0, 1.0, 2.0, 3.0, 4.0, 1.0]);
    assert_eq!(m.det(), -456.0);
    let m = Mat4f::from_slice_rowmajor(&[1.0, 1.0, 2.0, 3.0, 0.0, 1.0, 6.0, 7.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 0.0, 1.0]);
    assert_eq!(m.det(), 1.0);
  }
}
