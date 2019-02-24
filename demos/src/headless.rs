extern crate nobs_vulkanism as vk;

mod make_sequence {
  vk::pipes::pipeline! {
    include = ["global", "everything"],
    //dump = "src/make_sequence.dump",
    //dset_name[0] = "Dset",

    stage = {
      ty = "comp",
      glsl = "src/make_sequence.comp",
      //spv = "src/a.spv",
      include = ["src", "other"],
    }
  }

  pub struct ub {
    pub num_elems: u32,
    pub i_first: u32,
    pub i_step: u32,
  }
}

pub fn main() {
  let lib = vk::VkLib::new();
  let inst = vk::instance::new()
    .validate(vk::DEBUG_REPORT_ERROR_BIT_EXT | vk::DEBUG_REPORT_WARNING_BIT_EXT)
    .application("awesome app", 0)
    .add_extension(vk::KHR_SURFACE_EXTENSION_NAME)
    .add_extension(vk::KHR_XLIB_SURFACE_EXTENSION_NAME)
    .create(lib)
    .unwrap();

  let (pdevice, device) = vk::device::PhysicalDevice::enumerate_all(inst.handle)
    .remove(0)
    .into_device()
    .add_extension(vk::KHR_SWAPCHAIN_EXTENSION_NAME)
    .add_queue(vk::device::QueueProperties {
      present: false,
      graphics: true,
      compute: true,
      transfer: true,
    })
    .create()
    .unwrap();

  let p = make_sequence::new(device.handle).create().unwrap();

  let mut pool = vk::pipes::DescriptorPool::with_capacity(device.handle, &make_sequence::SIZES, make_sequence::NUM_SETS).unwrap();
  let ds = pool.new_dset(p.dsets[&0].layout, &p.dsets[&0].sizes).unwrap();

  let mut allocator = vk::mem::Allocator::new(pdevice.handle, device.handle);

  let mut buf_ub = vk::NULL_HANDLE;
  let mut buf_out = vk::NULL_HANDLE;
  vk::mem::Buffer::new(&mut buf_ub)
    .size(std::mem::size_of::<make_sequence::ub>() as vk::DeviceSize)
    .usage(vk::BUFFER_USAGE_TRANSFER_DST_BIT | vk::BUFFER_USAGE_UNIFORM_BUFFER_BIT)
    .devicelocal(false)
    .new_buffer(&mut buf_out)
    .size(123 * std::mem::size_of::<u32>() as vk::DeviceSize)
    .usage(vk::BUFFER_USAGE_TRANSFER_DST_BIT | vk::BUFFER_USAGE_STORAGE_BUFFER_BIT)
    .devicelocal(false)
    .bind(&mut allocator, vk::mem::BindType::Scatter)
    .unwrap();

  make_sequence::dset::write(device.handle, ds)
    .ub(|b| b.buffer(buf_ub))
    .b_out(|b| b.buffer(buf_out))
    .update();

  let cpool = vk::cmd::Pool::new(device.handle, device.queues[0].family).unwrap();

  {
    let mapped = allocator.get_mapped(buf_ub).unwrap();
    let ubb = make_sequence::ub {
      num_elems: 123,
      i_first: 0,
      i_step: 1,
    };
    mapped.host_to_device(&ubb);
  }
  {
    let mapped = allocator.get_mapped(buf_ub).unwrap();
    let mut ubb: make_sequence::ub = unsafe { std::mem::uninitialized() };
    mapped.device_to_host(&mut ubb);
  }

  use vk::cmd::commands::*;
  let cs = cpool
    .begin_stream()
    .unwrap()
    .push(&BindPipeline::compute(p.handle))
    .push(&BindDset::new(vk::PIPELINE_BIND_POINT_COMPUTE, p.layout, 0, ds))
    .push(&Dispatch::xyz(1, 1, 1));

  //let batch = vk::cmd::BatchSubmit::new(device.handle).unwrap();
  let mut batch = vk::cmd::AutoBatch::new(device.handle).unwrap();
  batch.push(cs).submit(device.queues[0].handle).0.sync().unwrap();

  //    .begin(device.queues[0])
  //    .unwrap()
  //    .push(&BindPipeline::compute(p.handle))
  //    .push(&BindDset::new(vk::PIPELINE_BIND_POINT_COMPUTE, p.layout, 0, ds))
  //    .push(&Dispatch::xyz(1, 1, 1))
  //    .submit_immediate();

  {
    let mapped = allocator.get_mapped_region(buf_out, 0, 100 * 4).unwrap();
    println!("{:?}", mapped);
    let v = mapped.as_slice::<u32>();
    println!("{:?}", v);
  }
}
