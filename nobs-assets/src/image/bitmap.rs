use vkm::Vec2u;

/// Image encoded as a bitmap
///
/// The bitmap assumes one byte per channel.
/// No format is imposed on the bitmap.
#[derive(Default, Clone)]
pub struct Bitmap {
  data: Vec<u8>,
  size: Vec2u,
  // TODO: make this to a channel desc
  bpp: u8,
}

#[allow(dead_code)]
impl Bitmap {
  /// Create a new bitmap
  ///
  /// # Arguments
  ///  - 'size' - the size (width + height) in pixels
  ///  - 'bpp' - bytes per pixel
  pub fn new(size: Vec2u, bpp: u8) -> Self {
    let mut data = Vec::with_capacity(size.x as usize * size.y as usize * bpp as usize);
    data.resize(data.capacity(), 0);

    Self { data, size, bpp }
  }

  /// Flips the image on the y axis
  pub fn flip_y(&mut self) {
    let tmp = self.clone();

    let line = self.size.x as usize * self.bpp as usize;
    for y in 0..self.size.y as isize {
      unsafe {
        std::ptr::copy_nonoverlapping(
          tmp.data.as_ptr().offset(y * line as isize),
          self.data.as_mut_ptr().offset((self.size.y as isize - y) * line as isize),
          line,
        );
      }
    }
  }

  /// Swaps red and blue channel
  ///
  /// The image does not impose a format, so this method will allways
  /// swap the channel 0 and 2.
  /// If the images
  pub fn swap_red_blue(&mut self) {
    if self.bpp > 3 {
      for p in 0..self.size.y * self.size.x {
        let pix = self.pos_mut(p as usize);
        let t = pix[0];
        pix[0] = pix[2];
        pix[2] = t;
      }
    }
  }

  /// Size of the image in pixel
  pub fn size(&self) -> Vec2u {
    self.size
  }

  /// Bytes per pixel
  pub fn bpp(&self) -> u8 {
    self.bpp
  }
  /// Bits per pixel (`== bpp * 8`)
  pub fn bitspp(&self) -> u8 {
    self.bpp * 8
  }

  /// Access to raw data
  pub fn data(&self) -> &[u8] {
    &self.data
  }
  /// Mutable access to raw data
  pub fn data_mut(&mut self) -> &mut [u8] {
    &mut self.data
  }

  /// Pixel at positon
  pub fn pos(&self, p: usize) -> &[u8] {
    let p = p * self.bpp as usize;
    &self.data[p..p + self.bpp as usize]
  }
  /// Pixel at positon
  pub fn pos_mut(&mut self, p: usize) -> &mut [u8] {
    let p = p * self.bpp as usize;
    &mut self.data[p..p + self.bpp as usize]
  }

  /// Pixel at coordinate
  pub fn pixel(&self, p: Vec2u) -> &[u8] {
    self.pos((p.y * self.size.x + p.x) as usize)
  }
  /// Pixel at coordinate
  pub fn pixel_mut(&mut self, p: Vec2u) -> &mut [u8] {
    self.pos_mut((p.y * self.size.x + p.x) as usize)
  }
}
