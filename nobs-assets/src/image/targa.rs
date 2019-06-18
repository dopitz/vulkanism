
use super::bitmap;
use std::fs;
use std::io;
use std::io::prelude::*;

#[allow(dead_code)]
pub enum ImageTypeCode {
  NoData = 0,
  Indexed = 1,
  RGB = 2,
  Grayscale = 3,
  RLEIndexed = 9,
  RLERGB = 10,
  RLEGrayscale = 11,
}

#[repr(C)]
#[derive(Default, Debug)]
#[allow(dead_code)]
pub struct Header {
  pub id_length: u8,
  pub colormap_type: u8,
  pub image_type_code: u8,
  pub colormap_spec: [u8; 5],
  pub x_origin: u16,
  pub y_origin: u16,
  pub width: u16,
  pub height: u16,
  pub bpp: u8,
  pub image_desc: u8,
}

impl Header {
  fn read(f: &mut fs::File) -> Result<Self, String> {
    let mut buffer = Vec::with_capacity(std::mem::size_of::<Header>());
    buffer.resize(buffer.capacity(), 0);

    f.read(&mut buffer).map_err(|_e| "read error")?;

    let mut header = Header::default();
    unsafe {
      std::ptr::copy_nonoverlapping(buffer.as_ptr() as *const Header, &mut header, 1);
    }
    Ok(header)
  }

  pub fn is_supported(&self) -> bool {
    // TODO: targa: support greyscale
    self.image_type_code == ImageTypeCode::RGB as u8 || self.image_type_code == ImageTypeCode::RLERGB as u8
  }
  pub fn is_compressed(&self) -> bool {
    self.image_type_code >= ImageTypeCode::RLEIndexed as u8
  }
}

pub struct Targa {
  pub img: bitmap::Bitmap,
}

impl Targa {
  pub fn load(name: &str) -> Result<Self, String> {
    let mut f = fs::File::open(name).map_err(|_e| "could not open file")?;
    let header = Header::read(&mut f)?;

    if !header.is_supported() {
      Err("unsupported targa")?
    }

    if header.id_length > 0 {
      f.seek(io::SeekFrom::Current(header.id_length as i64)).map_err(|_e| "read error")?;
    }

    let img = bitmap::Bitmap::new(vec2!(header.width, header.height).into(), header.bpp / 8);
    match header.is_compressed() {
      true => Self::load_compressed(img, f),
      false => Self::load_uncompressed(img, f),
    }
  }

  fn load_compressed(mut img: bitmap::Bitmap, mut f: fs::File) -> Result<Self, String> {
    let bpp = img.bpp() as usize;
    let mut byte = 0;
    let mut chunk = Vec::with_capacity(128 * bpp);
    chunk.resize(chunk.capacity(), 0);

    while byte < img.data().len() {
      f.read(&mut chunk[0..1]).map_err(|_e| "read error")?;

      // read 'count' consecutive pixels
      if chunk[0] < 128 {
        let count = chunk[0] as usize;
        let chunk = &mut chunk[0..count * bpp];
        f.read(chunk).map_err(|_e| "read error")?;

        for (src, dst) in chunk.iter().zip(img.data_mut().iter_mut().skip(byte)) {
          *dst = *src;
        }
        byte += chunk.len();
      }
      // repeat pixel 'count' times
      else {
        let count = chunk[0] as usize - 127;
        let chunk = &mut chunk[0..bpp];
        f.read(chunk).map_err(|_e| "read error")?;

        for _i in 0..count {
          for b in 0..bpp {
            img.data_mut()[byte] = chunk[b];
            byte += 1;
          }
        }

        byte += count * bpp;
      }
    }

    img.swap_red_blue();
    Ok(Self { img })
  }

  fn load_uncompressed(mut img: bitmap::Bitmap, mut f: fs::File) -> Result<Self, String> {
    f.read(img.data_mut()).map_err(|_e| "read error")?;
    img.swap_red_blue();

    Ok(Self { img })
  }
}
