use crate::sprites;
use vk::builder::Buildable;
use vk::pipes::DescriptorPool;
use vk::pipes::Pipeline;

pub struct PipePool {
  pub pipe: Pipeline,
  pub pool: DescriptorPool,
}

pub struct Pipelines {
  pub sprites: PipePool,
  pub shared_pool: DescriptorPool,
  pub ds_viewport: vk::DescriptorSet,
}

impl Pipelines {
  pub fn new(device: vk::Device, pass: vk::RenderPass, subpass: u32, ub_viewport: vk::Buffer) -> Self {
    let sprites = {
      let pipe = sprites::Pipeline::create_pipeline(device, pass, subpass);
      let pool = DescriptorPool::new(device, DescriptorPool::new_capacity().add(&pipe.dsets[1], 32));
      PipePool { pipe, pool }
    };

    let shared_pool = DescriptorPool::new(device, DescriptorPool::new_capacity().add(&sprites.pipe.dsets[0], 1)); // dset with ub_viewport
    let ds_viewport = shared_pool.new_dset(&sprites.pipe.dsets[0]).unwrap();
    vk::pipes::descriptor::writes::Writes::new(device, ds_viewport)
      .buffer(
        0,
        0,
        vk::DESCRIPTOR_TYPE_UNIFORM_BUFFER,
        vk::DescriptorBufferInfo::build().buffer(ub_viewport).into(),
      )
      .update();

    Self {
      sprites,
      shared_pool,
      ds_viewport,
    }
  }
}
