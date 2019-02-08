# vulkanism
Low level vulkan for rust

This library provides a low level access to the vulkan api. It is composed of several crates, namely
 - nobs-vk: Wrapps the vulkan api in a mest basic fashion, by only defining constants and types and setting up function entry points for core and extension commands.
 - nobs-vkcmd: Handles command buffers and their synchronisation
 - nobs-vkmem: Handles alloctation of device memory and binding resources to it
 - nobs-pipes: Handles pipelines and descriptor sets, compiles and generates rust code for shaders form glsl and spv
 - nobs-vkfb: Handles creation of framebuffer and renderpasses
 - nobs-wnd: Handles creation of a vulkan drawable window and swapchain

These crates share no dependencies with each other, exept that all depend on nobs-vk. This is designed to be able to pick and choose only the parts you want/like.

For ease of access three more crates are made available:
 - nobs-vulkanism: bundles all submodules into one dependency
 - nobs-vulkanism-headless: bundels all submodules exept nobs-wnd into one dependency (for offscreen rendering)
 - nobs-vulkanism-compute: bundels all submodules exept nobs-fb and nobs-wnd into one dependency (for compute tasks)

 This is usefull if you which not to haul in the full stack of window creation, when you only need compute capability.


## Setup
For a detailed setup destription follow the setup link for every submodule.

