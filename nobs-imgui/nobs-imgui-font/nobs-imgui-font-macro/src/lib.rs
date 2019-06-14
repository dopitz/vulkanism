extern crate freetype;
extern crate nobs_vkmath as vkm;
extern crate proc_macro;

use std::collections::HashMap;
use std::io::Write;
use std::process::Command;

use proc_macro::TokenStream;
use proc_macro::TokenTree;

use vkm::*;

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

struct FontBuilder {
  path: String,
  char_height: usize,
  mip_levels: usize,
  chars: Vec<char>,
  dump: Option<String>,
}

impl FontBuilder {
  pub fn from_tokens(input: TokenStream) -> Result<Self, String> {
    let mut font = String::new();
    let mut char_height = 64;
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
        std::env::var("CARGO_MANIFEST_DIR").unwrap().to_owned() + "/" + &font
      }
    };

    // no duplicate chars
    chars.sort();
    chars.dedup();

    // height in pixels for rendering the characters and number of mipmap levels
    // no point in going lower than 4 pixel height per character
    let (char_height, mip_levels) = {
      let mut i = 0;
      while (4 << i) < char_height {
        i += 1;
      }
      (4 << i, i + 1)
    };

    Ok(Self {
      path,
      char_height,
      mip_levels,
      chars,
      dump,
    })
  }

  fn load_chars(&self) -> Result<(freetype::Library, freetype::Face, Vec<Char>), String> {
    let lib = freetype::Library::init().map_err(|_| "Could not init freetype")?;
    let face = lib.new_face(&self.path, 0).map_err(|_| "Could not load face")?;
    face.set_pixel_sizes(0, self.char_height as u32).unwrap();

    let chars = self
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

    Ok((lib, face, chars))
  }

  fn compute_texture_dimensions(&self, chars: &[Char]) -> (Vec2s, Vec2s) {
    // we store characters in a regular grid
    // this saves us the headache when generating mipmaps
    let mut slot_size = chars.iter().fold(vec2!(0), |acc, c| Vec2::max(acc, c.size));
    let d = 1 << self.mip_levels;
    let m = slot_size.y % d;
    if d != 0 {
      slot_size.x = slot_size.x + d - m
    }

    // try to make as square as possible
    let mut tex_slots = vec2!(1);
    while tex_slots.x * tex_slots.y < chars.len() {
      let size = tex_slots * slot_size;
      let tex_aspect = size.x as f32 / size.y as f32;
      if tex_aspect > 1.0 {
        tex_slots.y += 1;
      } else {
        tex_slots.x += 1;
      }
    }

    (slot_size, tex_slots * slot_size)
  }

  fn make_image(&self, face: &freetype::Face, chars: &[Char], slot_size: Vec2s, tex_size: Vec2s) -> (Vec<u8>, HashMap<char, TexChar>) {
    let mut data: Vec<u8> = Vec::with_capacity(tex_size.x * tex_size.y);
    data.resize(data.capacity(), 0);

    let mut set = |at: Vec2s, v| {
      data[at.y * tex_size.x + at.x] = v;
    };

    let mut offset = vec2!(0);
    let mut tex_chars = HashMap::new();
    let mut y_bearing_max = 0.0;

    for c in chars.iter() {
      face.load_char(c.c as usize, freetype::face::LoadFlag::RENDER).unwrap();
      let g = face.glyph();
      let gbm = face.glyph().bitmap();
      let char_size = vec2!(gbm.width(), gbm.rows()).into();

      // copy char
      for y in 0..char_size.y {
        for x in 0..char_size.x {
          set(offset + vec2!(x, y), gbm.buffer()[(char_size.y - y - 1) * gbm.pitch() as usize + x]);
        }
      }

      let tc = offset.into::<f32>() / tex_size.into();

      // store char position inside the texture
      tex_chars.entry(c.c).or_insert(TexChar {
        tex00: tc,
        tex11: tc + char_size.into() / tex_size.into(),
        size: char_size.into() / self.char_height as f32,
        bearing: vec2!(g.bitmap_left(), -g.bitmap_top()).into() / self.char_height as f32,
        advance: vec2!(g.advance().x >> 6, g.advance().y >> 6).into() / self.char_height as f32,
      });

      offset.x += slot_size.x;
      if offset.x >= tex_size.x {
        offset.x = 0;
        offset.y += slot_size.y;
      }

      y_bearing_max = f32::max(g.bitmap_top() as f32 / self.char_height as f32, y_bearing_max);
    }

    // fix the origin of the glyph into the bearing from top left corner
    for (_, c) in tex_chars.iter_mut() {
      c.bearing.y -= 1.0 - y_bearing_max;
    }

    (data, tex_chars)
  }

  pub fn create(&self) -> Result<String, String> {
    let (_, face, chars) = self.load_chars()?;
    let (slot_size, tex_size) = self.compute_texture_dimensions(&chars);
    let (data, chars) = self.make_image(&face, &chars, slot_size, tex_size);

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

    let char_tbl = (0..256).into_iter().map(|c| c.to_string() + ", ").collect::<Vec<String>>();
    let s = format!(
      "
      pub fn new(device: vk::Device, mut mem: vk::mem::Mem, copy_queue: vk::Queue, cmds: &vk::cmd::CmdPool) -> Font {{
        use vk::cmd::stream::*;

        let mut tex = vk::NULL_HANDLE;
        vk::mem::Image::new(&mut tex)
          .texture2d({dimx}, {dimy}, vk::FORMAT_R8_UNORM)
          .mip_levels({mip_levels})
          .bind(&mut mem.alloc, vk::mem::BindType::Scatter)
          .unwrap();

        let texview = vk::ImageViewCreateInfo::build()
          .texture2d(tex, vk::FORMAT_R8_UNORM)
          .mip_levels(0, {mip_levels})
          .create(device)
          .unwrap();

        let sampler = vk::SamplerCreateInfo::build()
          .min_filter(vk::FILTER_LINEAR)
          .mag_filter(vk::FILTER_LINEAR)
          .mipmap_mode(vk::SAMPLER_MIPMAP_MODE_LINEAR)
          .min_lod(0.0)
          .max_lod({mip_levels} as f32)
          .create(device)
          .unwrap();

        let chars = CHARS.iter().fold(std::collections::HashMap::new(), |mut acc, (c, cp)| {{acc.entry(*c).or_insert(*cp); acc}});

        let mut stage = vk::mem::Staging::new(mem.clone(), ({dimx} * {dimy}) as vk::DeviceSize).unwrap();
        let mut map = stage.map().unwrap();
        let data = map.as_slice_mut::<u8>();

        unsafe {{
          std::ptr::copy_nonoverlapping(TEX.as_ptr(), data.as_mut_ptr(), data.len());
        }}

        let mut cs = cmds.begin_stream().unwrap().push(
          &stage.copy_into_image(
            tex,
            vk::BufferImageCopy::build()
              .image_extent(vk::Extent3D::build().set({dimx}, {dimy}, 1).into())
              .subresource(vk::ImageSubresourceLayers::build().aspect(vk::IMAGE_ASPECT_COLOR_BIT).into()),
          ),
        );

        let (mut w, mut h) = ({dimx}, {dimy});

        for l in 1..{mip_levels} {{
          cs = cs
            .push(&vk::cmd::commands::ImageBarrier::to_transfer_src(tex).mip_level(l-1, 1))
            .push(&vk::cmd::commands::ImageBarrier::to_transfer_dst(tex).mip_level(l, 1))
            .push(
              &vk::cmd::commands::Blit::new()
                .src(tex)
                .src_offset_end(w, h, 1)
                .src_subresource(vk::ImageSubresourceLayers::build().aspect(vk::IMAGE_ASPECT_COLOR_BIT).mip_level(l-1).into())
                .dst(tex)
                .dst_subresource(vk::ImageSubresourceLayers::build().aspect(vk::IMAGE_ASPECT_COLOR_BIT).mip_level(l).into())
                .dst_offset_end(w / 2, h / 2, 1)
              );
              w /= 2;
              h /= 2;
        }}
        cs = cs.push(&vk::cmd::commands::ImageBarrier::to_shader_read(tex).mip_level(0, {mip_levels}));

        

        let mut batch = vk::cmd::AutoBatch::new(device).unwrap();
        batch.push(cs).submit(copy_queue).0.sync().unwrap();

        Font::new(device, mem, tex, texview, sampler, chars)
      }}

      const CHARS : &[(char, Char)] = &[{chars}];
    
      const TEX_DIM : vkm::Vec2u = vkm::Vec2 {{ x: {dimx}, y: {dimy} }};
      const TEX : &[u8] = &[{tex}];
      ",
      mip_levels = self.mip_levels,
      chars = chars
        .iter()
        .fold(String::new(), |s, (c, p)| format!("{} ({:?}, {}), ", s, c, fmt_char(p))),
      dimx = tex_size.x,
      dimy = tex_size.y,
      //tex = data.iter().fold(String::new(), |s, c| format!("{} {}, ", s, c))
      tex = data.iter().fold(String::with_capacity(tex_size.x * tex_size.y * 5), |mut s, c| {
        s.push_str(&char_tbl[*c as usize]);
        s
      }),
    );

    Ok(s)
  }
}
