extern crate nobs_vk as vk;
extern crate nobs_vkmem as vkmem;

#[derive(Debug)]
struct Ub {
  a: u32,
  b: u32,
  c: u32,
}

fn main() {
  let lib = vk::VkLib::new();
  let inst = vk::instance::new()
    .validate(vk::DEBUG_REPORT_ERROR_BIT_EXT | vk::DEBUG_REPORT_WARNING_BIT_EXT)
    .application("awesome app", 0)
    .add_extension(vk::KHR_SURFACE_EXTENSION_NAME)
    .add_extension(vk::KHR_XLIB_SURFACE_EXTENSION_NAME)
    .create(lib)
    .expect("instance creation failed");

  let (pdevice, device) = vk::device::PhysicalDevice::enumerate_all(inst.handle)
    .remove(0)
    .into_device()
    .add_queue(vk::device::QueueProperties {
      present: false,
      graphics: true,
      compute: true,
      transfer: true,
    })
    .create()
    .expect("device creation failed");

  // create an allocator with default page size (128MiB for device local / 8MB for host visible memory)
  let mut allocator = vkmem::Allocator::new(pdevice.handle, device.handle);

  // declare handles
  let mut buf_ub = vk::NULL_HANDLE;
  let mut buf_out = vk::NULL_HANDLE;
  let mut img = vk::NULL_HANDLE;
  let mut bb = vk::NULL_HANDLE;

  // after resources have been created we bind them to the allocator
  // this will automatically allocate memory (if necessary)
  // bind_many should be preferred, since it tries to minimize the number of continuous memory blocks on which the resources are bound
  // this is only important if resources have been deleted and free blocks of memory are distributed over multiple pages
  // memory can not be rearranged after deleting/unbinding a resource, since vulkan does not allow rebinding a buffer/image to a different location
  //
  // to bind a single resource the allocator needs the handle and memory propetries (bundled in a vkmem::BindInfo)
  // it is possible to additionally specify the size of the resource in bytes
  // this only matters if the resource is a buffer that will be mapped into host accessible memory
  // if such a buffer was not bound with it's actual size, the size of the corresponding vk::MemoryRequirements is used,
  // which may contain a padding after the actual buffer
  //
  // vkmem::BindInfo::new(vkmem::Handle::Buffer(buf), properties)
  // allocator.bind(vkmem::Handle::Buffer(buf_ub));
  // allocator.bind_many(&[vkmem::Handle::Buffer(buf_out), vkmem::Handle::Image(img)]);

  // the builders vkmem::Buffer and vkmem::Image will atomatically take care of specifying the correct BindInfo to the Allocator
  // after one buffer has been finished configuring, we add another resource to be created with next_buffer / next_image
  // when bind as called all buffers/images will be created and bound to memory in bulk to as less different pages as possible
  // this is beneficial, if resources should be bound to the same memory and next to each other
  //
  // it is also possible to allocate resources on a dedicated page with bind_minipage
  // this will always allocate a new page with the exact size that all specified BindInfos need
  //vkmem::Buffer::new(&mut buf_ub)
  vkmem::Resource::new()
    .new_buffer(&mut buf_ub)
    .size(std::mem::size_of::<Ub>() as vk::DeviceSize)
    .usage(vk::BUFFER_USAGE_TRANSFER_DST_BIT | vk::BUFFER_USAGE_UNIFORM_BUFFER_BIT)
    .devicelocal(false)
    .new_buffer(&mut buf_out)
    .size(123 * std::mem::size_of::<u32>() as vk::DeviceSize)
    .usage(vk::BUFFER_USAGE_TRANSFER_DST_BIT | vk::BUFFER_USAGE_STORAGE_BUFFER_BIT)
    .devicelocal(false)
    .new_image(&mut img)
    .image_type(vk::IMAGE_TYPE_2D)
    .size(123, 123, 1)
    .usage(vk::IMAGE_USAGE_SAMPLED_BIT)
    .devicelocal(true)
    .new_buffer(&mut bb)
    .size(123 * std::mem::size_of::<u32>() as vk::DeviceSize)
    .usage(vk::BUFFER_USAGE_TRANSFER_DST_BIT | vk::BUFFER_USAGE_STORAGE_BUFFER_BIT)
    .devicelocal(true)
    .bind(&mut allocator, vkmem::BindType::Scatter)
    .unwrap();

  // Mapped gives a convenient view on the memory
  // get_mapped mapps the whole block of memory to which the resources was bound
  // get_mapped_region lets us define a byte offset respective to the beginning of the resource and a size in bytes
  {
    let mapped = allocator.get_mapped(buf_ub).unwrap();
    let ubb = Ub { a: 123, b: 4, c: 5 };
    mapped.host_to_device(&ubb);
  }
  {
    let mapped = allocator.get_mapped(buf_ub).unwrap();
    let mut ubb = Ub { a: 0, b: 0, c: 0 };
    mapped.device_to_host(&mut ubb);
    println!("{:?}", ubb);
  }

  {
    let mapped = allocator.get_mapped_region(buf_out, 4, 100).unwrap();
    println!("{:?}", mapped);
    let v = mapped.as_slice::<u32>();
    println!("{:?}", v);
  }

  // we can print stats in a yaml format for the currently allocated pages
  println!("{}", allocator.print_stats());

  // buffers and images can be destroyed
  // destroy_many should be preferred, because this will rearrange the datastructure to find free blocks of memory
  //allocator.destroy(img);
  allocator.destroy_many(&[buf_ub, buf_out]);
  println!("{}", allocator.print_stats());

  // destroying does NOT free memory
  // if we want to free memory we can do this only if a whole page is not used any more
  // in this case we can free the memory again
  allocator.free_unused();

  // dropping the allocator automatically destroys bound resources and frees all memory
}
