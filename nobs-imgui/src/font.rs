use vk;
use vk::builder::Buildable;
use vk::cmd;
use vk::mem;

use crate::ImGui;

use freetype;

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

#[derive(Clone, Copy)]
pub struct Font {
  pub tex: vk::Image,
  pub texview: vk::ImageView,
  pub sampler: vk::Sampler,
}

impl Font {
  pub fn new(_font: &FontID, gui: &ImGui) -> Self {
    let margin = 32;
    let target_size = 64;

    let mut tex = vk::NULL_HANDLE;
    mem::Image::new(&mut tex)
      .texture2d(target_size, target_size, vk::FORMAT_R8_UNORM)
      .bind(&mut gui.alloc.clone(), mem::BindType::Scatter)
      .unwrap();

    let texview = vk::ImageViewCreateInfo::build()
      .texture2d(tex, vk::FORMAT_R8_UNORM)
      .create(gui.device)
      .unwrap();

    let sampler = vk::SamplerCreateInfo::build().min_filter(vk::FILTER_LINEAR).mag_filter(vk::FILTER_LINEAR).create(gui.device).unwrap();

    // Init the library
    let lib = freetype::Library::init().unwrap();
    // Load a font face
    let face = lib.new_face("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf", 0).unwrap();
    // Set the font size
    //face.set_char_size(0, 64 * 2000, 0, 100).unwrap();
    face.set_pixel_sizes(0, target_size * 5).unwrap();
    // Load a character
    face.load_char('g' as usize, freetype::face::LoadFlag::RENDER).unwrap();
    // Get the glyph instance
    let glyph = face.glyph();

    println!(
      "{:?} {:?}   {:?} {:?} {:?}",
      glyph.bitmap().width(),
      glyph.bitmap().rows(),
      glyph.bitmap_left(),
      glyph.bitmap_top(),
      glyph.advance().x >> 6,
    );

    {
      let stage = mem::Staging::new(&mut gui.alloc.clone(), (target_size * target_size) as vk::DeviceSize).unwrap();
      let mut map = stage.range(0, (target_size * target_size) as vk::DeviceSize).map().unwrap();
      let data = map.as_slice_mut::<u8>();

      //unsafe {
      //  std::ptr::copy_nonoverlapping(glyph.bitmap().buffer().as_ptr(), data.as_mut_ptr(), data.len());
      //}

      for d in data.iter_mut() {
        *d = 0;
      }

      let bm = &glyph.bitmap();
      let bm_size = vec2!(bm.width(), bm.rows());
      //let glyph_margin = vec2!(margin, margin);
      //let glyph_sizem = glyph_size + glyph_margin;
      //let glyph_tex_offset = margin / target_size as f32;

      let bm_size_m = vec2!(i32::max(bm_size.x, bm_size.y) + 2 * margin);

      let sample_bm = |p: vkm::Vec2i| {
        if p.x >= 0 && p.y >= 0 && p.x < bm_size.x && p.y < bm_size.y {
          let pix = p.into::<usize>();
          glyph.bitmap().buffer()[pix.y * bm.pitch() as usize + pix.x] > 0
        } else {
          false
        }
      };

      for y in 0..target_size {
        for x in 0..target_size {
          let tex = vec2!(1.0) / target_size as f32 * vec2!(x, y).into();

          let pix = (bm_size_m.into() * tex).into::<i32>() - vec2!(margin);

          let s = sample_bm(pix);

          let mut d = margin as f32;
          for bmy in pix.y - margin..pix.y + margin {
            for bmx in pix.x - margin..pix.x + margin {
              if s != sample_bm(vec2!(bmx, bmy)) {
                d = f32::min(d, vkm::Vec2f::len(pix.into() - vec2!(bmx, bmy).into()));
              }
            }
          }

          if s {
            //data[y as usize * 64 + x as usize] = 255;
            //data[y as usize * 64 + x as usize] = 255 - (255.0 * (d / margin as f32)) as u8;
            data[y as usize * 64 + x as usize] = (255.0 * (0.5 + d / 2.0 / margin as f32)) as u8;
          }
          else {
            data[y as usize * 64 + x as usize] = (255.0 * (0.5 - d / 2.0 / margin as f32)) as u8;
          }

        }
      }

      //for y in 0..64 {
      //  for x in 0..64 {
      //    if glyph.bitmap().buffer()[y * glyph.bitmap().pitch() as usize + x as usize] > 0 {
      //      data[y * 64 + x] = 255;
      //    }
      //  }
      //}

      //let data = map.as_slice_mut::<u8>();
      //for y in 0..glyph.bitmap().rows() {
      //  unsafe {
      //    std::ptr::copy_nonoverlapping(
      //      //glyph.bitmap().buffer().as_ptr().offset((y * glyph.bitmap().pitch()) as isize),
      //      xx.as_ptr(),
      //      data.as_mut_ptr().offset((y * 256 * 4) as isize),
      //      glyph.bitmap().pitch() as usize,
      //    );
      //  }
      //}

      let cs = gui.cmds.begin_stream().unwrap().push(
        &stage.copy_into_image(
          tex,
          vk::BufferImageCopy::build()
            .image_extent(vk::Extent3D::build().set(target_size, target_size, 1).extent)
            .subresource(vk::ImageSubresourceLayers::build().aspect(vk::IMAGE_ASPECT_COLOR_BIT).layers),
        ),
      );

      let mut batch = cmd::AutoBatch::new(gui.device).unwrap();
      batch.push(cs).submit(gui.queue_copy).0.sync().unwrap();
    }

    //    {
    //      let stage = mem::Staging::new(&mut gui.alloc.clone(), 256 * 256 * 4).unwrap();
    //      let mut map = stage
    //        .range(0, 256 * 256 * std::mem::size_of::<u32>() as vk::DeviceSize)
    //        .map()
    //        .unwrap();
    //      let data = map.as_slice_mut::<u32>();
    //
    //      for d in data.iter_mut() {
    //        *d = 0xFF << 24 | 0xFF << 8 | 0xFF;
    //      }
    //
    //      let cs = gui.cmds.begin_stream().unwrap().push(
    //        &stage.copy_into_image(
    //          tex,
    //          vk::BufferImageCopy::build()
    //            .image_extent(vk::Extent3D::build().set(256, 256, 1).extent)
    //            .subresource(vk::ImageSubresourceLayers::build().aspect(vk::IMAGE_ASPECT_COLOR_BIT).layers),
    //        ),
    //      );
    //
    //      let mut batch = cmd::AutoBatch::new(gui.device).unwrap();
    //      batch.push(cs).submit(gui.queue_copy).0.sync().unwrap();
    //    }

    Font { tex, texview, sampler }
  }
}
