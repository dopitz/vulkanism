use vk;

#[derive(Debug, PartialEq, Clone)]
pub struct Binding {
  pub name: String,
  pub binding: u32,
  pub descset: u32,
  pub desctype: vk::DescriptorType,
  pub stageflags: vk::ShaderStageFlagBits,
}

impl Binding {
  pub fn to_binding_string(&self) -> String {
    format!(
      "Binding {{ name: &\"{name}\", binding: {binding}, descset: {descset}, desctype: {desctype}, stageflags: {stageflags} }},\n",
      name = self.name,
      binding = self.binding,
      descset = self.descset,
      desctype = self.desctype,
      stageflags = self.stageflags,
    )
  }

  pub fn to_dset_write_string(&self, dset_name: &str) -> String {
    match self.desctype {
      vk::DESCRIPTOR_TYPE_UNIFORM_BUFFER
      | vk::DESCRIPTOR_TYPE_STORAGE_BUFFER
      | vk::DESCRIPTOR_TYPE_UNIFORM_BUFFER_DYNAMIC
      | vk::DESCRIPTOR_TYPE_STORAGE_BUFFER_DYNAMIC => format!(
        "
        pub fn {name}<F: Fn(DescriptorBufferInfoBuilder) -> DescriptorBufferInfoBuilder>(&'a mut self, f: F) -> &'a mut {dset_name} {{
          self.inner.push_buffer({binding}, 0, {ty}, f(DescriptorBufferInfoBuilder::default()).get());
          self
        }}
        ",
        dset_name = dset_name,
        name = self.name,
        binding = self.binding,
        ty = self.desctype
      ),

      vk::DESCRIPTOR_TYPE_SAMPLER
      | vk::DESCRIPTOR_TYPE_COMBINED_IMAGE_SAMPLER
      | vk::DESCRIPTOR_TYPE_SAMPLED_IMAGE
      | vk::DESCRIPTOR_TYPE_STORAGE_IMAGE => format!(
        "
        pub fn {name}<F: Fn(DescriptorImageInfoBuilder) -> DescriptorImageInfoBuilder>(&'a mut self, f: F) -> &'a mut {dset_name} {{
          self.inner.push_buffer({binding}, 0, {ty}, f(DescriptorImageInfoBuilder::default()).get());
          self
        }}
        ",
        dset_name = dset_name,
        name = self.name,
        binding = self.binding,
        ty = self.desctype
      ),

      vk::DESCRIPTOR_TYPE_UNIFORM_TEXEL_BUFFER | vk::DESCRIPTOR_TYPE_STORAGE_TEXEL_BUFFER => format!(
        "
        pub fn {name}(&'a mut self, view: BufferView) -> &'a mut {dset_name} {{
          self.inner.push_buffer({binding}, 0, {ty}, view);
          self
        }}
        ",
        dset_name = dset_name,
        name = self.name,
        binding = self.binding,
        ty = self.desctype,
      ),

      _ => panic!("unrecognized descriptor type"),
    }
  }
}
