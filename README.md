# vulkanism
Low level vulkan for rust

This library provides a low level access to the vulkan api. It is composed of several base crates, namely
 - nobs-vk: Wrapps the vulkan api in a mest basic fashion, by only defining constants and types and setting up function entry points for core and extension commands.
 - nobs-vkmem: Handles alloctation of device memory and binding resources to it
 - nobs-pipes: Handles pipelines and descriptor sets, compiles and generates rust code for shaders form glsl and spv

These crates share no dependencies with each other. This is designed to be able to pick and choose only the parts you want/like.

For ease of access two more crates are made available:
 - nobs-vulkanism-headless: bundles above crate into a single dependency and adds modules for comand buffer synchronisation and framebuffer management
 - nobs-vulkanism: extendss nobs-vulkanism-headless to rendering to an output window

 This is usefull if you do not whish to haul in the full stack of window creation, when you only need compute capability or offscreen rendering.

## Setup
For a detailed setup destription follow the setup link for every submodule.

