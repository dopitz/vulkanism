use crate::pipeline::builder;
use crate::pipeline::Binding;
use crate::pipeline::Pipeline;
use vk;

/// Builder for a compute pipeline
///
/// Configures the pipeline's bindings and shader stages.
///
/// For a successfull pipeline creation the builder needs to be constructed from a valid device and
/// valid bindings and a compute shader stage need to be configured.
pub struct Compute<'a> {
  device: &'a vk::DeviceExtensions,
  bindings: Vec<Binding>,
  comp: Option<vk::PipelineShaderStageCreateInfo>,
}

impl<'a> Compute<'a> {
  /// Create builder from a device
  ///
  /// The builder is initialized with no bindings and no compute stage configured
  pub fn from_device(device: &vk::DeviceExtensions) -> Compute {
    Compute {
      device,
      bindings: Default::default(),
      comp: None,
    }
  }

  /// Configures the bindings for the pieline.
  ///
  /// From the configured bindings the builder is able to create the descriptor set layouts and pipeline layout.
  pub fn bindings(&mut self, bindings: &[Binding]) -> &mut Self {
    self.bindings = bindings.to_vec();
    self
  }

  /// Configures the compute shader stage for the pipeline.
  pub fn comp(&mut self, comp: &vk::PipelineShaderStageCreateInfo) -> &mut Self {
    self.comp = Some(*comp);
    self
  }

  /// Create the pipeline from the current configuration
  pub fn new(self) -> Result<Pipeline, String> {
    let stage = self.comp.ok_or("No compute shader stage set")?;
    if stage.module == vk::NULL_HANDLE {
      Err("invalid module handle for compute stage")?
    }

    let (dsets, layout) = builder::create_layouts(self.device, &self.bindings);

    let create_info = vk::ComputePipelineCreateInfo {
      sType: vk::STRUCTURE_TYPE_COMPUTE_PIPELINE_CREATE_INFO,
      pNext: std::ptr::null(),
      flags: 0,
      stage: stage,
      layout: layout,
      basePipelineHandle: vk::NULL_HANDLE,
      basePipelineIndex: 0,
    };

    let mut handle = vk::NULL_HANDLE;
    vk::CreateComputePipelines(
      self.device.get_handle(),
      vk::NULL_HANDLE,
      1,
      &create_info,
      std::ptr::null(),
      &mut handle,
    );

    vk::DestroyShaderModule(self.device.get_handle(), stage.module, std::ptr::null());

    Ok(Pipeline {
      device: self.device.get_handle(),
      handle,
      dsets,
      layout,
    })
  }
}
