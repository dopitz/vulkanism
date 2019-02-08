pub mod compute;
pub mod graphics;

use crate::descriptor::DsetLayout;
use crate::pipeline::Binding;
use std::collections::HashMap;
use vk;

fn create_descriptor_layout(device: &vk::DeviceExtensions, bindings: &[Binding]) -> vk::DescriptorSetLayout {
  let layout_bindings: Vec<vk::DescriptorSetLayoutBinding> = bindings
    .iter()
    .map(|b| vk::DescriptorSetLayoutBinding {
      binding: b.binding,
      descriptorType: b.desctype,
      descriptorCount: 1,
      stageFlags: b.stageflags,
      pImmutableSamplers: std::ptr::null(),
    })
    .collect();

  let create_info = vk::DescriptorSetLayoutCreateInfo {
    sType: vk::STRUCTURE_TYPE_DESCRIPTOR_SET_LAYOUT_CREATE_INFO,
    pNext: std::ptr::null(),
    flags: 0,
    bindingCount: layout_bindings.len() as u32,
    pBindings: layout_bindings.as_ptr(),
  };

  let mut handle = vk::NULL_HANDLE;
  vk::CreateDescriptorSetLayout(device.get_handle(), &create_info, std::ptr::null(), &mut handle);
  handle
}

fn create_pipeline_layout(device: &vk::DeviceExtensions, dset_layouts: &[vk::DescriptorSetLayout]) -> vk::PipelineLayout {
  // create the pipeline layout
  let create_info = vk::PipelineLayoutCreateInfo {
    sType: vk::STRUCTURE_TYPE_PIPELINE_LAYOUT_CREATE_INFO,
    pNext: std::ptr::null(),
    flags: 0,
    setLayoutCount: dset_layouts.len() as u32,
    pSetLayouts: dset_layouts.as_ptr(),
    pushConstantRangeCount: 0,
    pPushConstantRanges: std::ptr::null(),
  };

  let mut handle = vk::NULL_HANDLE;
  vk::CreatePipelineLayout(device.get_handle(), &create_info, std::ptr::null(), &mut handle);
  handle
}

fn create_pool_sizes(bindings: &[Binding]) -> Vec<vk::DescriptorPoolSize> {
  let counts = bindings.iter().fold(std::collections::HashMap::new(), |mut acc, b| {
    *acc.entry(b.desctype).or_insert(0u32) += 1;
    acc
  });

  counts
    .into_iter()
    .map(|(k, v)| vk::DescriptorPoolSize {
      typ: k,
      descriptorCount: v,
    })
    .collect()
}

fn create_layouts(device: &vk::DeviceExtensions, bindings: &[Binding]) -> (HashMap<u32, DsetLayout>, vk::PipelineLayout) {
  // spilt up bindings by descriptor set
  let dset_bindings = bindings.iter().fold(HashMap::new(), |mut acc, b| {
    {
      let v = acc.entry(b.descset).or_insert(Vec::new());
      v.push(*b);
    }
    acc
  });

  // layout and sizes for every descriptor set
  let dsets: HashMap<u32, DsetLayout> = dset_bindings
    .iter()
    .map(|(set, b)| {
      (
        *set,
        DsetLayout {
          layout: create_descriptor_layout(device, b),
          sizes: create_pool_sizes(b),
        },
      )
    })
    .collect();

  // pipeline layout
  let layouts: Vec<vk::DescriptorSetLayout> = dsets.values().map(|ds| ds.layout).collect();
  let pipe_layout = create_pipeline_layout(device, &layouts);

  (dsets, pipe_layout)
}
