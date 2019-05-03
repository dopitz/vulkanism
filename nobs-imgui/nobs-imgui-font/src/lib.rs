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

#[derive(Clone)]
struct TexChar {
  tex: vkm::Vec2f,
  size: vkm::Vec2f,
  bearing: vkm::Vec2f,
  advance: vkm::Vec2f,
}

struct FontBuilder {
  path: String,
  margin: usize,
  char_height: usize,
  chars: Vec<char>,
  dump: Option<String>,
}

impl FontBuilder {
  pub fn from_tokens(input: TokenStream) -> Result<Self, String> {
    let mut font = String::new();
    let mut margin = 0;
    let mut char_height = 0;
    let mut chars = (32u32..127u32).map(|c| std::char::from_u32(c).unwrap()).collect::<Vec<_>>();
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
        PATHS
          .iter()
          .map(|p| (*p).to_owned() + "/" + &font)
          .find(|p| std::path::Path::new(&p).is_file())
          .ok_or(format!("Could not find font: {}", font))?
      }
    };

    chars.sort();
    chars.dedup();

    Ok(Self {
      path,
      margin,
      char_height,
      chars,
      dump,
    })
  }

  pub fn create(&self) -> Result<String, String> {
    let margin = self.margin;
    let char_height = self.char_height * 64;

    // Init the library
    let lib = freetype::Library::init().map_err(|_| "Could not init freetype")?;
    // Load a font face
    let face = lib.new_face(&self.path, 0).map_err(|_| "Could not load face")?;
    face.set_pixel_sizes(0, char_height as u32).unwrap();

    let char_height = char_height + margin * 2;

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

    let (bm, chars) = Bitmap::from_chars(4096, self.margin, self.char_height * 64, &chars, &face);
    let bm = Arc::new(bm);

    let tex_size = bm.size / 16;
    let tex_bm = Arc::new(Mutex::new(Bitmap::new(tex_size)));

    #[derive(Clone)]
    struct Block {
      margin: i32,
      bm: Arc<Bitmap>,
      out: Arc<Mutex<Bitmap>>,
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
        out.data[self.ys * pitch..self.ye * pitch].copy_from_slice(&data.data);
      }
    }

    let n_threads = 8;
    let mut blocks = (0..n_threads)
      .map(|i| {
        let bs = tex_size.y / n_threads;
        let ys = i * bs;
        let ye = if i == n_threads - 1 { tex_size.y } else { ys + bs };

        (Block {
          margin: margin as i32,
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


    // generate code
    let pixel_size = vec2!(1.0) / bm.size.into();
    let fmt_vec = |v: vkm::Vec2f| format!("vkm::Vec2 {{ x: {:.15}, y: {:.15} }}", v.x, v.y);
    let fmt_char = |c: &TexChar| {
      format!(
        "Char {{ tex: {}, size: {}, bearing: {}, advance: {} }}",
        fmt_vec(c.tex * pixel_size),
        fmt_vec(c.size * pixel_size),
        fmt_vec(c.bearing * pixel_size),
        fmt_vec(c.advance * pixel_size),
      )
    };

    let s = format!(
      "
      pub fn new(gui: &ImGui) {{
      }}

      const CHARS : &[(char, Char)] = &[{}];
      
      const TEX : &[u8] = &[{}];
      ",
      chars.iter().fold(String::new(), |s, (c, p)| format!("{} ({:?}, {}), ", s, c, fmt_char(p))),
      tex_bm
        .lock()
        .unwrap()
        .data
        .iter()
        .fold(String::new(), |s, c| format!("{} {}, ", s, c))
    );

    Ok(s)

    //{
    //  let stage = mem::Staging::new(&mut gui.alloc.clone(), (tex_size.x * tex_size.y) as vk::DeviceSize).unwrap();
    //  let mut map = stage.range(0, (tex_size.x * tex_size.y) as vk::DeviceSize).map().unwrap();
    //  let data = map.as_slice_mut::<u8>();

    //  unsafe {
    //    std::ptr::copy_nonoverlapping(tex_data.lock().unwrap().as_ptr(), data.as_mut_ptr(), data.len());
    //  }

    //  let cs = gui.cmds.begin_stream().unwrap().push(
    //    &stage.copy_into_image(
    //      tex,
    //      vk::BufferImageCopy::build()
    //        .image_extent(vk::Extent3D::build().set(tex_size.x, tex_size.y, 1).extent)
    //        .subresource(vk::ImageSubresourceLayers::build().aspect(vk::IMAGE_ASPECT_COLOR_BIT).layers),
    //    ),
    //  );

    //  let mut batch = cmd::AutoBatch::new(gui.device).unwrap();
    //  batch.push(cs).submit(gui.queue_copy).0.sync().unwrap();
    //}

    //Font {
    //  tex,
    //  texview,
    //  sampler,
    //  chars: char_pos,
    //  char_height: 5.0,
    //}
  }
}

#[derive(Clone)]
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
    pitch: usize,
    margin: usize,
    char_height: usize,
    chars: &[Char],
    face: &freetype::Face,
  ) -> (Self, HashMap<char, TexChar>) {
    let mut bm = Vec::new();
    let mut tex_chars = HashMap::new();

    let mut char_height = char_height + 2 * margin;
    let mut size = vec2!(0);
    let mut to = vec2!(0);
    for c in chars.iter().rev() {
      face.load_char(c.c as usize, freetype::face::LoadFlag::RENDER).unwrap();
      let g = face.glyph();
      let gbm = face.glyph().bitmap();
      let char_size = vec2!(gbm.width(), gbm.rows()).into();

      // new line in bitmap / resize bitmap
      if to.x + char_size.x >= pitch || to.y + char_height >= size.y {
        let lines = bm.len() / (pitch * char_height) + 1;
        bm.resize(pitch * char_height * lines, 0u8);

        size.x = usize::max(size.x, to.x);
        size.y += char_height;

        to.x = 0;
        to.y = (lines - 1) * char_height;
      }

      // copy char
      for y in 0..char_size.y {
        for x in 0..char_size.x {
          bm[(to.y + margin + (char_size.y - y)) * pitch + to.x + margin + x] = gbm.buffer()[y * gbm.pitch() as usize + x];
        }
      }

      // store char position in texture
      tex_chars.entry(c.c).or_insert(TexChar {
        tex: to.into(),
        size: (char_size + vec2!(margin) * 2).into(),
        bearing: vec2!(g.bitmap_left(), -g.bitmap_top()).into(),
        advance: vec2!(g.advance().x >> 6, g.advance().y >> 6).into(),
      });

      // advance
      to.x += char_size.x;
    }

    to.y += char_height;

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
