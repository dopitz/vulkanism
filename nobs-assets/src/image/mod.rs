mod bitmap;
mod targa;

pub use bitmap::Bitmap;

use crate::Update;
use vk;
use vk::builder::Buildable;
use vk::cmd::commands::ImageBarrier;

struct AssetType {}

impl crate::AssetType for AssetType {
  type Type = vk::Image;
  fn load(id: &str, up: &mut Update) -> Self::Type {
    let tga = targa::Targa::load(id).unwrap();

    let mut tex = vk::NULL_HANDLE;
    vk::mem::Image::new(&mut tex)
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
      .bind(&mut up.mem.alloc, vk::mem::BindType::Block)
      .unwrap();

    let mut stage = up.get_staging(tga.img.data().len() as vk::DeviceSize);
    stage.map().unwrap().host_to_device_slice(tga.img.data());

    up.push_image((
      stage.copy_into_image(
        tex,
        vk::BufferImageCopy::build()
          .subresource(vk::ImageSubresourceLayers::build().aspect(vk::IMAGE_ASPECT_COLOR_BIT).into())
          .image_extent(vk::Extent3D::build().set(tga.img.size().x, tga.img.size().y, 1).into()),
      ),
      Some(ImageBarrier::to_shader_read(tex)),
    ));

    tex
  }

  fn free(asset: Self::Type, up: &mut Update) {
    up.mem.trash.push_image(asset);
  }
}
