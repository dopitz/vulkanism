mod bitmap;
mod targa;

pub use bitmap::Bitmap;

use crate::Update;
use vk;
use vk::builder::Buildable;
use vk::cmd::commands::ImageBarrier;

pub struct Asset {
  handle: vk::Image,
}

impl crate::Asset for Asset {
  type Id = String;

  fn load(id: &Self::Id, up: &mut Update) -> Self {
    let tga = targa::Targa::load(id).unwrap();

    let mut handle = vk::NULL_HANDLE;
    vk::mem::Image::new(&mut handle)
      .texture2d(
        tga.img.size().x,
        tga.img.size().y,
        match tga.img.bpp() {
          1 => vk::FORMAT_R8_UNORM,
          3 => vk::FORMAT_R8G8B8_UNORM,
          4 => vk::FORMAT_R8G8B8A8_UNORM,
          _ => panic!("invalid texture format"),
        },
      )
      .bind(&mut up.get_mem().alloc, vk::mem::BindType::Block)
      .unwrap();

    let mut stage = up.get_staging(tga.img.data().len() as vk::DeviceSize);
    stage.map().unwrap().host_to_device_slice(tga.img.data());

    up.push_image(
      stage.copy_into_image(
        handle,
        vk::BufferImageCopy::build()
          .subresource(vk::ImageSubresourceLayers::build().aspect(vk::IMAGE_ASPECT_COLOR_BIT).into())
          .image_extent(vk::Extent3D::build().set(tga.img.size().x, tga.img.size().y, 1).into()),
      ),
      Some(ImageBarrier::to_shader_read(handle)),
    );

    Self { handle }
  }

  fn free(self, up: &mut Update) {
    up.get_mem().trash.push_image(self.handle);
  }
}
