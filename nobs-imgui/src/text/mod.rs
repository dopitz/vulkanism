use std::sync::Arc;
use std::sync::Weak;

use vk;
use vk::builder::Buildable;
use vk::cmd;
use vk::mem;
use vk::pipes;

mod pipe {
  vk::pipes::pipeline! {
    stage = {
      ty = "vert",
      glsl = "src/text/text.vert",
    }

    stage = {
      ty = "frag",
      glsl = "src/text/text.frag",
    }
  }

  #[repr(C)]
  pub struct Vertex {
    pub pos: cgm::Vector4<f32>,
    pub tex: cgm::Vector2<f32>,
  }

//  use std::sync::Arc;
//  use std::sync::Weak;
//  pub static CACHED_PIPE: Weak<Option<vk::pipes::Pipeline>> = Weak::new();
}

pub struct Text {
  pub pipe: vk::pipes::Pipeline,
  pub dpool: vk::pipes::DescriptorPool,
  pub ds: vk::DescriptorSet,

  pub vb: vk::Buffer,
  pub ub: vk::Buffer,
  pub tex: vk::Image,
  pub texview: vk::ImageView,
  pub sampler: vk::Sampler,

  pub draw: cmd::commands::DrawVertices,
}

impl Drop for Text {
  fn drop(&mut self) {
    vk::DestroyImageView(self.pipe.device, self.texview, std::ptr::null());
    vk::DestroySampler(self.pipe.device, self.sampler, std::ptr::null());
  }
}

impl Text {
  pub fn new(
    device: &vk::device::Device,
    cmds: &vk::cmd::Pool,
    rp: &vk::fb::Renderpass,
    alloc: &mut vk::mem::Allocator,
    staging: &mut vk::mem::Staging,
  ) -> Self {
    let pipe = pipe::new(rp.device, rp.pass, 0)
      .input_assembly(vk::PipelineInputAssemblyStateCreateInfo::build().topology(vk::PRIMITIVE_TOPOLOGY_TRIANGLE_STRIP))
      .dynamic(
        vk::PipelineDynamicStateCreateInfo::build()
          .push_state(vk::DYNAMIC_STATE_VIEWPORT)
          .push_state(vk::DYNAMIC_STATE_SCISSOR),
      )
      .blend(vk::PipelineColorBlendStateCreateInfo::build().push_attachment(vk::PipelineColorBlendAttachmentState::build()))
      .create()
      .unwrap();

    let mut vb = vk::NULL_HANDLE;
    let mut ub = vk::NULL_HANDLE;
    let mut tex = vk::NULL_HANDLE;
    vk::mem::Buffer::new(&mut vb)
      .vertex_buffer(3 * std::mem::size_of::<pipe::Vertex>() as vk::DeviceSize)
      .new_buffer(&mut ub)
      .uniform_buffer(2 * std::mem::size_of::<f32>() as vk::DeviceSize)
      .new_image(&mut tex)
      .texture2d(256, 256, vk::FORMAT_R8G8B8A8_UNORM)
      .bind(alloc, vk::mem::BindType::Block)
      .unwrap();

    let texview = vk::ImageViewCreateInfo::build()
      .texture2d(tex, vk::FORMAT_R8G8B8A8_UNORM)
      .create(device.handle)
      .unwrap();
    let sampler = vk::SamplerCreateInfo::build().create(device.handle).unwrap();

    {
      let mut map = staging
        .range(0, 3 * std::mem::size_of::<pipe::Vertex>() as vk::DeviceSize)
        .map()
        .unwrap();
      let svb = map.as_slice_mut::<pipe::Vertex>();
      svb[0].pos = cgm::Vector4::new(0.0, 1.0, 0.0, 1.0);
      svb[0].tex = cgm::Vector2::new(1.0, 1.0);

      svb[1].pos = cgm::Vector4::new(-1.0, -1.0, 0.0, 1.0);
      svb[1].tex = cgm::Vector2::new(1.0, 1.0);

      svb[2].pos = cgm::Vector4::new(1.0, -1.0, 0.0, 1.0);
      svb[2].tex = cgm::Vector2::new(1.0, 1.0);

      let cs = cmds.begin_stream().unwrap().push(&staging.copy_into_buffer(vb, 0));

      let mut batch = vk::cmd::AutoBatch::new(cmds.device).unwrap();
      batch.push(cs).submit(device.queues[0].handle).0.sync().unwrap();
    }

    {
      let mut map = staging
        .range(0, 256 * 256 * std::mem::size_of::<u32>() as vk::DeviceSize)
        .map()
        .unwrap();
      let data = map.as_slice_mut::<u32>();

      for d in data.iter_mut() {
        *d = 0xFF << 24 | 0xFF << 8 | 0xFF;
      }

      let cs = cmds.begin_stream().unwrap().push(
        &staging.copy_into_image(
          tex,
          vk::BufferImageCopy::build()
            .image_extent(vk::Extent3D::build().set(256, 256, 1).extent)
            .subresource(vk::ImageSubresourceLayers::build().aspect(vk::IMAGE_ASPECT_COLOR_BIT).layers),
        ),
      );

      let mut batch = vk::cmd::AutoBatch::new(device.handle).unwrap();
      batch.push(cs).submit(device.queues[0].handle).0.sync().unwrap();
    }

    {
      let mut map = staging.range(0, 2 * std::mem::size_of::<u32>() as vk::DeviceSize).map().unwrap();
      let data = map.as_slice_mut::<u32>();
      data[0] = 1;
      data[1] = 1;

      let cs = cmds.begin_stream().unwrap().push(&staging.copy_into_buffer(ub, 0));

      let mut batch = vk::cmd::AutoBatch::new(cmds.device).unwrap();
      batch.push(cs).submit(device.queues[0].handle).0.sync().unwrap();
    }

    let mut dpool = vk::pipes::DescriptorPool::with_capacity(device.handle, &pipe::SIZES, pipe::NUM_SETS).unwrap();
    let ds = dpool.new_dset(pipe.dsets[&0].layout, &pipe.dsets[&0].sizes).unwrap();

    pipe::dset::write(device.handle, ds)
      .ub_viewport(|b| b.buffer(ub))
      //.tex_sampler(|s| s.set(vk::IMAGE_LAYOUT_SHADER_READ_ONLY_OPTIMAL, texview, sampler))
      .update();

    let draw = cmd::commands::Draw::default().push(vb, 0).vertices().vertex_count(4);

    Text {
      pipe,
      dpool,
      ds,

      vb,
      ub,
      tex,
      texview,
      sampler,

      draw,
    }
  }
}

impl cmd::commands::StreamPush for Text {
  fn enqueue(&self, cs: cmd::Stream) -> cmd::Stream {
    cs.push(&cmd::commands::BindPipeline::graphics(self.pipe.handle))
      .push(&cmd::commands::BindDset::new(
        vk::PIPELINE_BIND_POINT_GRAPHICS,
        self.pipe.layout,
        0,
        self.ds,
      ))
      .push(&self.draw)
  }
}
