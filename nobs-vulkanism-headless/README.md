# nobs-vkpipes
shader compilation and pipeline composing on top of nobs-vk.

This librarary provides codegeneration from glsl and spv sources at compiletime. Creates a rust module for a single shader or whole pipeline. The generated modules contain information about shader uniform bindings, names and descriptor sets. With this easy to access descriptor updates can be performed without any additional information from the shoder sources at runtime.

## Documentation
Find a complete documentation of this library at [docs.rs](https://docs.rs/nobs-vkpipes).

## Setup
Follow the setup instructions for [shaderc-rs](https://github.com/google/shaderc-rs).

Nothing more is to be done, you are ready to use nobs-vkpipes.

## Contributing
Feel encouraged to contribute!


