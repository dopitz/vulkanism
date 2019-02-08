# nobs-vkmem
Vulkan memory management as extension to [nobs-vk](https://github.com/dopitz/nobs-vk).

## Motivation
Buffer and image creation in vulkan is tricky in comparison to e.g. OpenGL, because
1. We have to create the buffer/image and then later bind it to a`vkDeviceMemory` that has to be created separately.
2. Creating a single `vkDeviceMemory` allocation for every buffer/image is bad practice, in fact it is [encouraged](https://developer.nvidia.com/vulkan-memory-management) to bind resources that are used together on the same allocation.
3. Another layer of difficulty is introduced with memory types, since not all resources (should) share the same memory properties - which is different for each Driver/Vendor

## Features
nobs-vkmem provides convenient and accessible methods for creating buffers and images and binding them to physical memory. This dramatically reduces boiler plate code, while still offers the user considerable control over how resources are bound to memory.

1. Easy buffer and image creation with builder patterns.
2. Device memory is allocated in larger pages. The library keeps track of free and used regions in a page.
3. Offers different allocation strategies for different purposes, including forcing the binding of several resources to a continuous block, or binding resources on private pages.
4. Easy mapping of host accessible buffers


## Documentation
Find a complete documentation of this library at [docs.rs](https://docs.rs/nobs-vkmem).

### Example Usage
```rust
  // create an allocator with default page size 
  // (128MiB for device local / 8MB for host visible memory)
  let mut allocator = vkmem::Allocator::new(physical_device_handle, device_handle);

  // declare handles
  let mut buf_ub = vk::NULL_HANDLE;
  let mut img = vk::NULL_HANDLE;
  let mut bb = vk::NULL_HANDLE;

  // configure
  vkmem::Buffer::new(&mut buf_ub)
    .size(std::mem::size_of::<Ub>() as vk::DeviceSize)
    .usage(vk::BUFFER_USAGE_TRANSFER_DST_BIT | vk::BUFFER_USAGE_UNIFORM_BUFFER_BIT)
    .devicelocal(false)
    .next_image(&mut img)
    .image_type(vk::IMAGE_TYPE_2D)
    .size(123, 123, 1)
    .usage(vk::IMAGE_USAGE_SAMPLED_BIT)
    .next_buffer(&mut bb)
    .size(123 * std::mem::size_of::<u32>() as vk::DeviceSize)
    .usage(vk::BUFFER_USAGE_TRANSFER_DST_BIT | vk::BUFFER_USAGE_STORAGE_BUFFER_BIT)
    // binds all configured resources in bulk using as less blocks of memory as possible.
    // But allows to split resources of the same memory type to be scattered to multiple blocks
    // on the same page, if no large enough free block is found
    .bind(&mut allocator, vkmem::BindType::Scatter)
    .unwrap();

  // Mapped gives a convenient view on the memory
  {
    let mapped = allocator.get_mapped(buf_ub).unwrap();
    let ubb = Ub { a: 123, b: 4, c: 5 };
    mapped.host_to_device(&ubb);
    // going out of scope automatically unmapps...
  }
  {
    let mapped = allocator.get_mapped(buf_ub).unwrap();
    let mut ubb = Ub { a: 0, b: 0, c: 0 };
    mapped.device_to_host(&mut ubb);
    println!("{:?}", ubb);
  }

  // we can print stats in a yaml format for the currently allocated pages
  println!("{}", allocator.print_stats());

  // buffers and images can be destroyed
  // destroy_many() should be preferred over destroy(),
  // because this will rearrange and merge allocated blocks into the free list again
  allocator.destroy_many(&[buf_ub, img]);

  // destroying does NOT free memory
  // if we want to free memory we can do this only, if a whole page is not used any more
  // in this case we can free the memory again
  allocator.free_unused();
```

## Contributing
Feel encouraged to contribute, in any way you can think!


