pub mod compute;
pub mod graphics;

use crate::pipeline::Binding;
use crate::DescriptorLayout;
use std::collections::HashMap;
use vk;

pub fn create_descriptor_layout(device: vk::Device, bindings: &[Binding]) -> vk::DescriptorSetLayout {
  let layout_bindings: Vec<vk::DescriptorSetLayoutBinding> = bindings
    .iter()
    .map(|b| vk::DescriptorSetLayoutBinding {
      binding: b.binding,
      descriptorType: b.desctype,
      descriptorCount: b.arrayelems,
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
  vk::CreateDescriptorSetLayout(device, &create_info, std::ptr::null(), &mut handle);
  handle
}

pub fn create_pipeline_layout(device: vk::Device, dset_layouts: &[vk::DescriptorSetLayout]) -> vk::PipelineLayout {
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
  vk::CreatePipelineLayout(device, &create_info, std::ptr::null(), &mut handle);
  handle
}

fn create_layouts(device: vk::Device, bindings: &[Binding]) -> (Vec<DescriptorLayout>, vk::PipelineLayout) {
  // spilt up bindings by descriptor set
  let dset_bindings = bindings.iter().fold(HashMap::new(), |mut acc, b| {
    {
      let v = acc.entry(b.descset).or_insert(Vec::new());
      v.push(*b);
    }
    acc
  });

  // layout and sizes for every descriptor set
  let mut dsets: Vec<(u32, DescriptorLayout)> = dset_bindings
    .iter()
    .map(|(set, b)| (*set, DescriptorLayout::from_bindings(device, b)))
    .collect();
  dsets.sort_by_key(|d| d.0);
  let dsets: Vec<DescriptorLayout> = dsets.iter().map(|ds| ds.1.clone()).collect();

  // pipeline layout
  let layouts: Vec<vk::DescriptorSetLayout> = dsets.iter().map(|ds| ds.layout).collect();
  let pipe_layout = create_pipeline_layout(device, &layouts);

  (dsets, pipe_layout)
}
