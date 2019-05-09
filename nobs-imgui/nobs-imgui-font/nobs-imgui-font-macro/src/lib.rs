extern crate freetype;
extern crate nobs_vkmath as vkm;
extern crate proc_macro;

use std::collections::HashMap;
use std::io::Write;
use std::path::Path;
use std::process::Command;
use std::sync::Arc;
use std::sync::Mutex;

use proc_macro::TokenStream;
use proc_macro::TokenTree;

use vkm::*;

const PATHS: &[&str] = &["/usr/share/fonts", "/usr/share/fonts/truetype"];

fn parse<T: Iterator<Item = TokenTree>>(tokens: &mut T) -> String {
  tokens
    .take_while(|tok| match tok {
      TokenTree::Punct(p) => p.to_string() != ",",
      _ => true,
    })
    .fold(String::new(), |s, tok| format!("{}{}", s, tok.to_string()))
}

macro_rules! proc_macro_err {
  ($ex:expr) => {
    match $ex {
      Err(e) => {
        return format!("compile_error!(\"{}\")", format!("{}", e).replace("\"", "\\\""))
          .parse()
          .unwrap();
      }
      Ok(x) => x,
    }
  };
}

#[proc_macro]
pub fn make_font(input: TokenStream) -> TokenStream {
  let b = proc_macro_err!(FontBuilder::from_tokens(input));
  let s = proc_macro_err!(b.create());

  if let Some(dump) = b.dump {
    let mut f = proc_macro_err!(std::fs::File::create(&dump).map_err(|_| format!("Could not open file: {}", dump)));
    proc_macro_err!(f.write_all(s.as_bytes()).map_err(|_| "Could not write shader module to file"));

    proc_macro_err!(Command::new("sh")
      .arg("-c")
      .arg(format!("rustfmt {}", dump))
      .output()
      .map_err(|_| "Could not format shader module file, is rustfmt installed?"));
  }

  s.parse().unwrap()
}

struct Char {
  c: char,
  size: vkm::Vec2s,
}

struct TexChar {
  tex00: vkm::Vec2f,
  tex11: vkm::Vec2f,
  size: vkm::Vec2f,
  bearing: vkm::Vec2f,
  advance: vkm::Vec2f,
}

struct Bitmap {
  pub data: Vec<u8>,
  pub pitch: usize,
  pub size: vkm::Vec2s,
}

impl Bitmap {
  pub fn sample(&self, p: vkm::Vec2i) -> bool {
    if p.x >= 0 && p.y >= 0 && p.x < self.size.x as i32 && p.y < self.size.y as i32 {
      self.data[p.y as usize * self.pitch + p.x as usize] > 0
    } else {
      false
    }
  }

  pub fn new(size: vkm::Vec2s) -> Self {
    let mut data = Vec::new();
    data.resize(size.x * size.y, 0);
    let pitch = size.x;
    Self { data, pitch, size }
  }

  pub fn from_chars(
    downsample: usize,
    pitch: usize,
    margin: usize,
    char_height: usize,
    chars: &[Char],
    face: &freetype::Face,
  ) -> (Self, HashMap<char, TexChar>) {
    let mut bm = Vec::new();
    let mut tex_chars = HashMap::new();

    let mut to = vec2!(0);
    let mut row_height = 0;

    let scale = vec2!(1.0 / (char_height) as f32);
    //let margin = (margin.into() * scale).into();

    //bm.resize(pitch * char_height, 0u8);
    for c in chars.iter().rev() {
      face.load_char(c.c as usize, freetype::face::LoadFlag::RENDER).unwrap();
      let g = face.glyph();
      let gbm = face.glyph().bitmap();
      let char_size = vec2!(gbm.width(), gbm.rows()).into();
      let margin = vec2!(margin);

      // new line in bitmap / resize bitmap
      if to.x + char_size.x + 2 * margin.x >= pitch {
        to.x = 0;
        to.y += row_height;
        row_height = 0;
      }
      if char_size.y + 2 * margin.y > row_height {
        bm.resize(bm.len() + pitch * (char_size.y + 2 * margin.y - row_height), 0u8);
        row_height = char_size.y + 2 * margin.y;
      }

      // copy char
      for y in 0..char_size.y {
        for x in 0..char_size.x {
          bm[(to.y + margin.y + (char_size.y - y)) * pitch + to.x + margin.x + x] = gbm.buffer()[y * gbm.pitch() as usize + x];
        }
      }

      // store char position in texture
      tex_chars.entry(c.c).or_insert(TexChar {
        tex00: to.into(),
        tex11: (to + char_size + margin * 2).into(),
        size: char_size.into(),
        bearing: vec2!(g.bitmap_left(), -g.bitmap_top()).into(),
        advance: vec2!(g.advance().x >> 6, g.advance().y >> 6).into(),
      });

      // advance
      to.x += char_size.x + margin.x * 2;
    }

    let size = vec2!(pitch, {
      let h = to.y + char_height;
      if h % downsample == 0 {
        h
      } else {
        h + downsample - h % downsample
      }
    });

    bm.resize(size.x * size.y, 0u8);

    for (x, c) in tex_chars.iter_mut() {
      c.tex00 = c.tex00 / size.into();
      c.tex11 = c.tex11 / size.into();
      c.size = c.size * scale;
      c.bearing = c.bearing * scale;
      c.advance = c.advance * scale;
    }

    (Self { data: bm, pitch, size }, tex_chars)
  }
}

impl std::ops::Index<vkm::Vec2s> for Bitmap {
  type Output = u8;
  fn index(&self, i: vkm::Vec2s) -> &u8 {
    self.data.index(i.y * self.pitch + i.x)
  }
}
impl std::ops::IndexMut<vkm::Vec2s> for Bitmap {
  fn index_mut(&mut self, i: vkm::Vec2s) -> &mut u8 {
    self.data.index_mut(i.y * self.pitch + i.x)
  }
}

struct FontBuilder {
  path: String,
  margin: usize,
  char_height: usize,
  downsample: usize,
  chars: Vec<char>,
  dump: Option<String>,
}

impl FontBuilder {
  pub fn from_tokens(input: TokenStream) -> Result<Self, String> {
    let mut font = String::new();
    let mut margin = 16;
    let mut char_height = 640;
    let mut chars = (32u32..127u32).map(|c| std::char::from_u32(c).unwrap()).collect::<Vec<_>>();
    let mut downsample = 16;
    let mut dump = None;

    let mut tokens = input.clone().into_iter();
    while let Some(tok) = tokens.next() {
      match tok {
        TokenTree::Ident(i) => {
          let s = i.to_string();

          tokens.next();
          match s.as_ref() {
            "font" => font = parse(&mut tokens).chars().skip(1).take_while(|c| *c != '\"').collect(),
            "margin" => margin = parse(&mut tokens).parse::<usize>().unwrap(),
            "char_height" => char_height = parse(&mut tokens).parse::<usize>().unwrap(),
            "chars" => chars.extend_from_slice(&parse(&mut tokens).chars().collect::<Vec<_>>()),
            "downsample" => downsample = parse(&mut tokens).parse::<usize>().unwrap(),
            "dump" => dump = Some(parse(&mut tokens).chars().skip(1).take_while(|c| *c != '\"').collect()),
            _ => Err(format!("Unexpected identifier: {}", s))?,
          }
        }
        TokenTree::Group(g) => Err(format!("Expected identifier, found {}", g.to_string()))?,
        TokenTree::Literal(l) => Err(format!("Expected identifier, found {}", l.to_string()))?,
        TokenTree::Punct(p) => Err(format!("Expected identifier, found {}", p.to_string()))?,
      }
    }

    let path = {
      if std::path::Path::new(&font).is_file() {
        font.to_owned()
      } else {
        std::env::var("CARGO_MANIFEST_DIR").unwrap().to_owned() + "/" + &font
      }
    };

    chars.sort();
    chars.dedup();

    Ok(Self {
      path,
      margin,
      char_height,
      downsample,
      chars,
      dump,
    })
  }

  fn load_chars(&self) -> Result<(freetype::Library, freetype::Face, Vec<Char>), String> {
    let lib = freetype::Library::init().map_err(|_| "Could not init freetype")?;
    let face = lib.new_face(&self.path, 0).map_err(|_| "Could not load face")?;
    face.set_pixel_sizes(0, self.char_height as u32).unwrap();

    let mut chars = self
      .chars
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

    Ok((lib, face, chars))
  }

  fn resample(&self, bm: Arc<Bitmap>, size: vkm::Vec2s) -> Arc<Bitmap> {
    let tex_bm = Arc::new(Mutex::new(Arc::new(Bitmap::new(size))));

    #[derive(Clone)]
    struct Block {
      margin: i32,
      bm: Arc<Bitmap>,
      out: Arc<Mutex<Arc<Bitmap>>>,
      ys: usize,
      ye: usize,
    }

    impl Block {
      pub fn comp_distance_field(&mut self) {
        let out_size = { self.out.lock().unwrap().size };
        let mut data = Bitmap::new(vec2!(out_size.x, self.ye - self.ys));

        for y in 0..data.size.y {
          for x in 0..data.size.x {
            let tex = vec2!(1.0) / out_size.into() * vec2!(x, y + self.ys).into();
            let pix = (self.bm.size.into() * tex).into();

            let s = self.bm.sample(pix);

            let mut d = self.margin as f32;
            let beg = pix - vec2!(self.margin);
            let end = pix + vec2!(self.margin);
            for y in beg.y..end.y {
              for x in beg.x..end.x {
                let p = vec2!(x, y);
                if s != self.bm.sample(p) {
                  d = f32::min(d, vkm::Vec2f::len(pix.into() - p.into()));
                }
              }
            }

            if s {
              data[vec2!(x, y)] = (255.0 * (0.5 + d / 2.0 / self.margin as f32)) as u8;
            } else {
              data[vec2!(x, y)] = (255.0 * (0.5 - d / 2.0 / self.margin as f32)) as u8;
            }
          }
        }

        let mut out = self.out.lock().unwrap();
        let pitch = out.pitch;
        Arc::get_mut(&mut out).unwrap().data[self.ys * pitch..self.ye * pitch].copy_from_slice(&data.data);
      }
    }

    let n_threads = 8;
    let mut blocks = (0..n_threads)
      .map(|i| {
        let bs = size.y / n_threads;
        let ys = i * bs;
        let ye = if i == n_threads - 1 { size.y } else { ys + bs };

        (Block {
          margin: self.margin as i32,
          bm: bm.clone(),
          out: tex_bm.clone(),
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

    let res = tex_bm.lock().unwrap().clone();
    res
  }

  pub fn create(&self) -> Result<String, String> {
    let (lib, face, chars) = self.load_chars()?;

    let (bm, chars) = Bitmap::from_chars(self.downsample, 4096, self.margin, self.char_height, &chars, &face);
    let bm = Arc::new(bm);

    let tex_bm = self.resample(bm.clone(), bm.size / self.downsample);

    // generate code
    let fmt_vec = |v: vkm::Vec2f| format!("vkm::Vec2 {{ x: {:.15}, y: {:.15} }}", v.x, v.y);
    let fmt_char = |c: &TexChar| {
      format!(
        "Char {{ tex00: {}, tex11: {}, size: {}, bearing: {}, advance: {} }}",
        fmt_vec(c.tex00),
        fmt_vec(c.tex11),
        fmt_vec(c.size),
        fmt_vec(c.bearing),
        fmt_vec(c.advance),
      )
    };

    let s = format!(
      "
      pub fn new(device: vk::Device, alloc: &vk::mem::Allocator, queue_copy: vk::Queue, cmds: &vk::cmd::Pool) -> Font {{
        let mut tex = vk::NULL_HANDLE;
        vk::mem::Image::new(&mut tex)
          .texture2d({dimx}, {dimy}, vk::FORMAT_R8_UNORM)
          .bind(&mut alloc.clone(), vk::mem::BindType::Scatter)
          .unwrap();

        let texview = vk::ImageViewCreateInfo::build()
          .texture2d(tex, vk::FORMAT_R8_UNORM)
          .create(device)
          .unwrap();

        let sampler = vk::SamplerCreateInfo::build()
          .min_filter(vk::FILTER_LINEAR)
          .mag_filter(vk::FILTER_LINEAR)
          .create(device)
          .unwrap();

        let chars = CHARS.iter().fold(std::collections::HashMap::new(), |mut acc, (c, cp)| {{acc.entry(*c).or_insert(*cp); acc}});

        let mut stage = vk::mem::Staging::new(&mut alloc.clone(), ({dimx} * {dimy}) as vk::DeviceSize).unwrap();
        let mut map = stage.map().unwrap();
        let data = map.as_slice_mut::<u8>();

        unsafe {{
          std::ptr::copy_nonoverlapping(TEX.as_ptr(), data.as_mut_ptr(), data.len());
        }}

        let cs = cmds.begin_stream().unwrap().push(
          &stage.copy_into_image(
            tex,
            vk::BufferImageCopy::build()
              .image_extent(vk::Extent3D::build().set({dimx}, {dimy}, 1).extent)
              .subresource(vk::ImageSubresourceLayers::build().aspect(vk::IMAGE_ASPECT_COLOR_BIT).layers),
          ),
        );

        let mut batch = vk::cmd::AutoBatch::new(device).unwrap();
        batch.push(cs).submit(queue_copy).0.sync().unwrap();

        Font {{
          tex,
          texview,
          sampler,
          chars,
          char_height: 5.0,
        }}
      }}

      const CHARS : &[(char, Char)] = &[{chars}];
      
      const TEX_DIM : vkm::Vec2u = vkm::Vec2 {{ x: {dimx}, y: {dimy} }};
      const TEX : &[u8] = &[{tex}];
      ",
      chars = chars
        .iter()
        .fold(String::new(), |s, (c, p)| format!("{} ({:?}, {}), ", s, c, fmt_char(p))),
      dimx = tex_bm.size.x,
      dimy = tex_bm.size.y,
      tex = tex_bm.data.iter().fold(String::new(), |s, c| format!("{} {}, ", s, c))
    );

    Ok(s)
  }
}
