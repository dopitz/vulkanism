use vk;
use vk::builder::Buildable;
use vk::cmd;
use vk::mem;

use crate::ImGui;

use freetype;

pub struct FontBuilder {
  name: String,
  margin: u32,
  char_size: u32,
}

impl FontBuilder {
  pub fn new(name: String) -> FontBuilder {
    Self {
      name,
      margin: 32,
      char_size: 64 * 5,
    }
  }

  pub fn create() {}
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct FontID {
  name: String,
  size: u32,
}

impl FontID {
  pub fn new(name: &str, size: u32) -> Self {
    Self {
      name: name.to_owned(),
      size,
    }
  }
}

#[derive(Clone, Copy, Debug)]
pub struct Char {
  pub tex: vkm::Vec2f,
  pub size: vkm::Vec2f,
  pub bearing: vkm::Vec2f,
  pub advance: vkm::Vec2f,
}

impl std::ops::Mul<f32> for Char {
  type Output = Char;
  fn mul(self, s: f32) -> Self {
    Char {
      size : self.size * s,
      bearing : self.bearing * s,
      advance : self.advance * s,
      .. self
    }
  }
}

pub struct Font {
  pub tex: vk::Image,
  pub texview: vk::ImageView,
  pub sampler: vk::Sampler,

  pub chars: std::collections::HashMap<char, Char>,
}

impl Font {
  pub fn new(_font: &FontID, gui: &ImGui) -> Self {
    let margin = 32;
    let char_size = 64;

    // Init the library
    let lib = freetype::Library::init().unwrap();
    // Load a font face
    let face = lib.new_face("/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf", 0).unwrap();
    // Set the font size
    //face.set_char_size(0, 64 * 2000, 0, 100).unwrap();
    face.set_pixel_sizes(0, char_size * 5).unwrap();

    let mut chars = (32u32..127u32).map(|c| std::char::from_u32(c).unwrap()).collect::<Vec<_>>();
    chars.dedup();
    chars.sort();

    struct Char {
      c: char,
      size: vkm::Vec2s,
    }

    let mut chars = chars
      .iter()
      .map(|&c| {
        face.load_char(c as usize, freetype::face::LoadFlag::RENDER).unwrap();
        let g = face.glyph();
        Char {
          c,
          size: vec2!(g.bitmap().width(), g.bitmap().rows()).into(),
        }
      })
      .collect::<Vec<_>>();

    chars.sort_by_key(|c| c.size.y);

    let bm_size = vec2!(4096);
    let mut bm = Vec::new();
    bm.resize(bm_size.x * bm_size.y, 0);

    let mut copy_char = |glyph_bm: &freetype::bitmap::Bitmap, to: vkm::Vec2s| {
      for y in 0..glyph_bm.rows() as usize {
        for x in 0..glyph_bm.width() as usize {
          bm[(to.y + margin + (glyph_bm.rows() as usize - y)) * bm_size.x + to.x + margin + x] = glyph_bm.buffer()[y * glyph_bm.pitch() as usize + x];
        }
      }
    };

    let mut char_pos = std::collections::HashMap::new();

    let mut tex_size = vec2!(0usize, 64 * 5 + 2 * margin);
    let mut to = vec2!(0usize);
    for c in chars.iter().rev() {
      face.load_char(c.c as usize, freetype::face::LoadFlag::RENDER).unwrap();
      let g = face.glyph();

      let char_width = g.bitmap().width() as usize + 2 * margin;
      let char_height = g.bitmap().rows() as usize + 2 * margin;

      if to.x + char_width >= bm_size.x {
        tex_size.x = usize::max(tex_size.x, to.x);
        tex_size.y += 64 * 5 + 2 * margin;

        to.x = 0;
        to.y += 64 * 5 + 2 * margin;
      }

      copy_char(&g.bitmap(), to);

      char_pos.entry(c.c).or_insert(crate::font::Char {
        tex: to.into(),
        size: vec2!(char_width, char_height).into(),
        bearing: vec2!(g.bitmap_left(), g.bitmap_top()).into(),
        advance: vec2!(g.advance().x >> 6, g.advance().y >> 6).into(),
      });

      to.x += char_width;
    }

    to.y += 64 * 5 + 2 * margin;

    let margin = 32;
    let bm_region = tex_size.into();
    let tex_size = bm_region / 16;
    let (tex, texview, sampler) = Self::create_texture(tex_size, gui);

    let mut tex_data = Vec::new();
    tex_data.resize(tex_size.x as usize * tex_size.y as usize, 0);
    let tex_data = std::sync::Arc::new(std::sync::Mutex::new(tex_data));

    #[derive(Clone)]
    struct Block {
      bm: std::sync::Arc<Vec<u8>>,
      bm_size: vkm::Vec2i,
      bm_region: vkm::Vec2i,
      margin: i32,

      data: std::sync::Arc<std::sync::Mutex<Vec<u8>>>,
      data_size: vkm::Vec2s,
      ys: usize,
      ye: usize,
    }

    impl Block {
      fn sample_bm(&self, p: vkm::Vec2i) -> bool {
        if p.x >= 0 && p.y >= 0 && p.x < self.bm_size.x && p.y < self.bm_size.y {
          self.bm[(p.y * self.bm_size.x + p.x) as usize] > 0
        } else {
          false
        }
      }

      pub fn comp_distance_field(&mut self) {
        let mut block_size_y = self.ye - self.ys;
        let mut data = Vec::new();
        data.resize(block_size_y * self.data_size.x as usize, 0);

        for y in 0..block_size_y {
          for x in 0..self.data_size.x {
            let tex = vec2!(1.0) / self.data_size.into() * vec2!(x, y + self.ys).into();
            let pix = (self.bm_region.into() * tex).into();

            let s = self.sample_bm(pix);

            let mut d = self.margin as f32;
            let beg = pix - vec2!(self.margin);
            let end = pix + vec2!(self.margin);
            for bmy in beg.y..end.y {
              for bmx in beg.x..end.x {
                let p = vec2!(bmx, bmy);
                if s != self.sample_bm(p) {
                  d = f32::min(d, vkm::Vec2f::len(pix.into() - p.into()));
                }
              }
            }

            if s {
              data[y * self.data_size.x as usize + x] = (255.0 * (0.5 + d / 2.0 / self.margin as f32)) as u8;
            } else {
              data[y * self.data_size.x as usize + x] = (255.0 * (0.5 - d / 2.0 / self.margin as f32)) as u8;
            }
          }
        }

        let mut out = self.data.lock().unwrap();
        out[self.ys * self.data_size.x..self.ye * self.data_size.x].copy_from_slice(&data);
      }
    }

    let bm = std::sync::Arc::new(bm);
    let n_threads = 8;
    let mut blocks = (0..n_threads)
      .map(|i| {
        let bs = tex_size.y as usize / n_threads;
        let ys = i * bs;
        let ye = if i == n_threads - 1 { tex_size.y as usize } else { ys + bs };

        (Block {
          bm: bm.clone(),
          bm_size: bm_size.into(),
          bm_region: bm_region.into(),
          margin,

          data: tex_data.clone(),
          data_size: tex_size.into(),
          ys,
          ye,
        })
      })
      .collect::<Vec<_>>();

    let mut handles = Vec::new();
    for b in blocks.iter_mut() {
      let mut b = b.clone();
      let h = std::thread::spawn(move || b.comp_distance_field());
      handles.push(h);
    }

    for h in handles {
      h.join();
    }

    let pixel_size = vec2!(1.0) / bm_region.into();
    for (_, c) in char_pos.iter_mut() {
      c.tex = c.tex * pixel_size;
      c.size = c.size * pixel_size;
      c.bearing = c.bearing * pixel_size;
      c.advance = c.advance * pixel_size;
    }

    for (c, cp) in char_pos.iter_mut() {
      println!("{} {:?}", c, cp);
    }

    {
      let stage = mem::Staging::new(&mut gui.alloc.clone(), (tex_size.x * tex_size.y) as vk::DeviceSize).unwrap();
      let mut map = stage.range(0, (tex_size.x * tex_size.y) as vk::DeviceSize).map().unwrap();
      let data = map.as_slice_mut::<u8>();

      unsafe {
        std::ptr::copy_nonoverlapping(tex_data.lock().unwrap().as_ptr(), data.as_mut_ptr(), data.len());
      }

      let cs = gui.cmds.begin_stream().unwrap().push(
        &stage.copy_into_image(
          tex,
          vk::BufferImageCopy::build()
            .image_extent(vk::Extent3D::build().set(tex_size.x, tex_size.y, 1).extent)
            .subresource(vk::ImageSubresourceLayers::build().aspect(vk::IMAGE_ASPECT_COLOR_BIT).layers),
        ),
      );

      let mut batch = cmd::AutoBatch::new(gui.device).unwrap();
      batch.push(cs).submit(gui.queue_copy).0.sync().unwrap();
    }

    Font {
      tex,
      texview,
      sampler,
      chars: char_pos,
    }
  }

  pub fn get(&self, c: char) -> Char {
    self.chars.get(&c).cloned().unwrap_or(Char {
      tex: vec2!(0.0),
      size: vec2!(0.0),
      bearing: vec2!(0.0),
      advance: vec2!(0.0),
    })
  }

  fn create_texture(size: vkm::Vec2u, gui: &ImGui) -> (vk::Image, vk::ImageView, vk::Sampler) {
    let mut tex = vk::NULL_HANDLE;
    mem::Image::new(&mut tex)
      .texture2d(size.x, size.y, vk::FORMAT_R8_UNORM)
      .bind(&mut gui.alloc.clone(), mem::BindType::Scatter)
      .unwrap();

    let texview = vk::ImageViewCreateInfo::build()
      .texture2d(tex, vk::FORMAT_R8_UNORM)
      .create(gui.device)
      .unwrap();

    let sampler = vk::SamplerCreateInfo::build()
      .min_filter(vk::FILTER_LINEAR)
      .mag_filter(vk::FILTER_LINEAR)
      .create(gui.device)
      .unwrap();

    (tex, texview, sampler)
  }
}
