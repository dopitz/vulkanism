use vk;

#[derive(Debug, PartialEq, Clone)]
pub struct Binding {
  pub name: String,
  pub binding: u32,
  pub descset: u32,
  pub desctype: vk::DescriptorType,
  pub arrayelems: u32,
  pub stageflags: vk::ShaderStageFlagBits,
}

impl Binding {
  pub fn to_binding_string(&self) -> String {
    format!(
      "Binding {{ name: &\"{name}\", binding: {binding}, descset: {descset}, desctype: {desctype}, arrayelems: {arrayelems}, stageflags: {stageflags} }},\n",
      name = self.name,
      binding = self.binding,
      descset = self.descset,
      desctype = self.desctype,
      arrayelems = self.arrayelems,
      stageflags = self.stageflags,
    )
  }

  fn get_desctype_str(&self) -> (&str, &str) {
    match self.desctype {
      vk::DESCRIPTOR_TYPE_UNIFORM_BUFFER
      | vk::DESCRIPTOR_TYPE_STORAGE_BUFFER
      | vk::DESCRIPTOR_TYPE_UNIFORM_BUFFER_DYNAMIC
      | vk::DESCRIPTOR_TYPE_STORAGE_BUFFER_DYNAMIC => ("buffer", "vk::DescriptorBufferInfo"),

      vk::DESCRIPTOR_TYPE_SAMPLER
      | vk::DESCRIPTOR_TYPE_COMBINED_IMAGE_SAMPLER
      | vk::DESCRIPTOR_TYPE_SAMPLED_IMAGE
      | vk::DESCRIPTOR_TYPE_STORAGE_IMAGE => ("image", "vk::DescriptorImageInfo"),

      vk::DESCRIPTOR_TYPE_UNIFORM_TEXEL_BUFFER | vk::DESCRIPTOR_TYPE_STORAGE_TEXEL_BUFFER => ("bufferview", "vk::BufferView"),

      _ => panic!("unrecognized descriptor type"),
    }
  }

  pub fn to_dset_write_string(&self, dset_name: &str) -> String {
    let (setter, desctype) = self.get_desctype_str();

    if self.arrayelems == 1 {
      format!(
        "
        pub fn {name}(&mut self, info: {desctype}) -> &mut {dset_name} {{
          self.inner.{setter}({binding}, 0, {ty}, info);
          self
        }}
        ",
        name = self.name,
        dset_name = dset_name,
        binding = self.binding,
        ty = self.desctype,
        setter = setter,
        desctype = desctype
      )
    } else {
      format!(
        "
        pub fn {name}_elem(&mut self, array_elem: u32, info: {desctype}) -> &mut {dset_name} {{
          self.inner.{setter}({binding}, array_elem, {ty}, info);
          self
        }}
        pub fn {name}(&mut self, infos: &[{desctype}]) -> &mut {dset_name} {{
          self.inner.{setter}s({binding}, 0, {ty}, infos);
          self
        }}
        ",
        name = self.name,
        dset_name = dset_name,
        binding = self.binding,
        ty = self.desctype,
        setter = setter,
        desctype = desctype
      )
    }
  }

  /// Comparecs everything for equality except `stageflags`
  pub fn same_stage(a: &Self, b: &Self) -> bool {
    a.name == b.name && a.binding == b.binding && a.descset == b.descset && a.desctype == b.desctype && a.arrayelems == b.arrayelems
  }
}
