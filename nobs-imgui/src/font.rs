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
    let mut tex = vk::NULL_HANDLE;
    mem::Image::new(&mut tex)
      .texture2d(256, 256, vk::FORMAT_R8G8B8A8_UNORM)
      .bind(&mut gui.alloc.clone(), mem::BindType::Scatter)
      .unwrap();

    let texview = vk::ImageViewCreateInfo::build()
      .texture2d(tex, vk::FORMAT_R8G8B8A8_UNORM)
      .create(gui.device)
      .unwrap();

    let sampler = vk::SamplerCreateInfo::build().create(gui.device).unwrap();

    // Init the library
    let lib = freetype::Library::init().unwrap();
    // Load a font face
    let face = lib.new_face("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf", 0).unwrap();
    // Set the font size
    face.set_char_size(40 * 64, 0, 50, 0).unwrap();
    // Load a character
    face.load_char('A' as usize, freetype::face::LoadFlag::RENDER).unwrap();
    // Get the glyph instance
    let glyph = face.glyph();

    println!(
      "{:?} {:?} {:?}",
      glyph.bitmap().width(),
      glyph.bitmap().rows(),
      glyph.bitmap().buffer().len()
    );

    {
      let stage = mem::Staging::new(&mut gui.alloc.clone(), 256 * 256 * 4 as vk::DeviceSize).unwrap();
      let mut map = stage.range(0, 256 * 256 * 4 as vk::DeviceSize).map().unwrap();
      let data = map.as_slice_mut::<u32>();

      //unsafe {
      //  std::ptr::copy_nonoverlapping(glyph.bitmap().buffer().as_ptr(), data.as_mut_ptr(), data.len());
      //}

      for d in data.iter_mut() {
        *d = 0xFF << 24 | 0xFF << 8 | 0xFF;
      }

      let mut xx: Vec<u8> = Vec::new();
      for i in 0..glyph.bitmap().pitch() {
        xx.push(255);
      }

      for y in 0..20 {
        for x in 0..20 {
          data[y * 256 + x] = 0xFF << 24 | 0xFF;
        }
      }

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
            .image_extent(vk::Extent3D::build().set(256, 256, 1).extent)
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
