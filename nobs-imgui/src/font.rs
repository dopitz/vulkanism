use vk;
use vk::builder::Buildable;
use vk::cmd;
use vk::mem;

use crate::ImGui;

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

    {
      let stage = mem::Staging::new(&mut gui.alloc.clone(), 256 * 256 * 4).unwrap();
      let mut map = stage
        .range(0, 256 * 256 * std::mem::size_of::<u32>() as vk::DeviceSize)
        .map()
        .unwrap();
      let data = map.as_slice_mut::<u32>();

      for d in data.iter_mut() {
        *d = 0xFF << 24 | 0xFF << 8 | 0xFF;
      }

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

    Font { tex, texview, sampler }
  }
}
