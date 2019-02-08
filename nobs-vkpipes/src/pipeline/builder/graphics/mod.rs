pub mod blend;
pub mod depth_stencil;
pub mod dynamic;
pub mod input_assembly;
pub mod multisample;
pub mod raster;
pub mod tesselation;
pub mod vertex_input;
pub mod viewport;

use crate::pipeline::builder;
use crate::pipeline::Binding;
use crate::pipeline::Pipeline;
use vk;

/// Builder for a graphics pipeline
///
/// Configures the pipeline's bindings, shader stages, and states.
///
/// For a successfull pipeline creation the builder needs to be constructed from a valid device and
/// valid bindings and at least a vertex and fragment shader stage need to be configured.
pub struct Graphics<'a> {
  device: &'a vk::DeviceExtensions,
  pass: vk::RenderPass,
  subpass: u32,

  bindings: Vec<Binding>,
  vert: Option<vk::PipelineShaderStageCreateInfo>,
  tesc: Option<vk::PipelineShaderStageCreateInfo>,
  tese: Option<vk::PipelineShaderStageCreateInfo>,
  geom: Option<vk::PipelineShaderStageCreateInfo>,
  frag: Option<vk::PipelineShaderStageCreateInfo>,

  vertex_input: vertex_input::Builder,
  input_assembly: input_assembly::Builder,
  tesselation: tesselation::Builder,
  viewport: viewport::Builder,
  raster: raster::Builder,
  multisample: multisample::Builder,
  depth_stencil: depth_stencil::Builder,
  blend: blend::Builder,
  dynamic: dynamic::Builder,
}

impl<'a> Graphics<'a> {
  /// Create builder from a device
  ///
  /// The builder is initialized with no bindings and no shader stages configured.
  /// The default states are initialized with the default constructor of their respective builder.
  pub fn from_pass(device: &vk::DeviceExtensions, pass: vk::RenderPass) -> Graphics {
    Graphics {
      device,
      pass,
      subpass: 0,

      bindings: Default::default(),
      vert: None,
      tesc: None,
      tese: None,
      geom: None,
      frag: None,

      vertex_input: Default::default(),
      input_assembly: Default::default(),
      tesselation: Default::default(),
      viewport: Default::default(),
      raster: Default::default(),
      multisample: Default::default(),
      depth_stencil: Default::default(),
      blend: Default::default(),
      dynamic: Default::default(),
    }
  }

  /// Configures the bindings for the pieline.
  ///
  /// From the configured bindings the builder is able to create the descriptor set layouts and pipeline layout.
  pub fn bindings(&mut self, bindings: Vec<Binding>) -> &mut Self {
    self.bindings = bindings;
    self
  }
  /// Configures the vertex shader stage for the pipeline.
  pub fn vert(&mut self, vert: &vk::PipelineShaderStageCreateInfo) -> &mut Self {
    self.vert = Some(*vert);
    self
  }
  /// Configures the tesselation control shader stage for the pipeline.
  pub fn tesc(&mut self, tesc: &vk::PipelineShaderStageCreateInfo) -> &mut Self {
    self.tesc = Some(*tesc);
    self
  }
  /// Configures the tesselation evaluation shader stage for the pipeline.
  pub fn tese(&mut self, tese: &vk::PipelineShaderStageCreateInfo) -> &mut Self {
    self.tese = Some(*tese);
    self
  }
  /// Configures the geometry shader stage for the pipeline.
  pub fn geom(&mut self, geom: &vk::PipelineShaderStageCreateInfo) -> &mut Self {
    self.geom = Some(*geom);
    self
  }
  /// Configures the fragment shader stage for the pipeline.
  pub fn frag(&mut self, frag: &vk::PipelineShaderStageCreateInfo) -> &mut Self {
    self.frag = Some(*frag);
    self
  }

  /// Configures the vertex input state for the pipeline.
  pub fn vertex_input<F: Fn(&mut vertex_input::Builder)>(&mut self, f: F) -> &mut Self {
    f(&mut self.vertex_input);
    self
  }
  /// Configures the input assembly state for the pipeline.
  pub fn input_assembly<F: Fn(&mut input_assembly::Builder)>(&mut self, f: F) -> &mut Self {
    f(&mut self.input_assembly);
    self
  }
  /// Configures the tesselation state for the pipeline.
  pub fn tesselation<F: Fn(&mut tesselation::Builder)>(&mut self, f: F) -> &mut Self {
    f(&mut self.tesselation);
    self
  }
  /// Configures the viewport for the pipeline.
  pub fn viewport<F: Fn(&mut viewport::Builder)>(&mut self, f: F) -> &mut Self {
    f(&mut self.viewport);
    self
  }
  /// Configures the raster state for the pipeline.
  pub fn raster<F: Fn(&mut raster::Builder)>(&mut self, f: F) -> &mut Self {
    f(&mut self.raster);
    self
  }
  /// Configures the multisample state for the pipeline.
  pub fn multisample<F: Fn(&mut multisample::Builder)>(&mut self, f: F) -> &mut Self {
    f(&mut self.multisample);
    self
  }
  /// Configures the blend state for the pipeline.
  pub fn blend<F: Fn(&mut blend::Builder)>(&mut self, f: F) -> &mut Self {
    f(&mut self.blend);
    self
  }
  /// Configures the dynamic state for the pipeline.
  pub fn dynamic<F: Fn(&mut dynamic::Builder)>(&mut self, f: F) -> &mut Self {
    f(&mut self.dynamic);
    self
  }

  /// Create the pipeline from the current configuration
  pub fn new(self) -> Result<Pipeline, String> {
    let stages = [self.vert, self.tesc, self.tesc, self.geom, self.frag]
      .iter()
      .filter(|s| s.is_some())
      .map(|s| s.unwrap())
      .collect::<Vec<_>>();
    if stages.iter().any(|s| s.module == vk::NULL_HANDLE) {
      Err("invalid module handle")?
    }

    let (dsets, layout) = builder::create_layouts(self.device, &self.bindings);

    let create_info = vk::GraphicsPipelineCreateInfo {
      sType: vk::STRUCTURE_TYPE_GRAPHICS_PIPELINE_CREATE_INFO,
      pNext: std::ptr::null(),
      flags: 0,
      stageCount: stages.len() as u32,
      pStages: stages.as_ptr(),
      pVertexInputState: self.vertex_input.get(),
      pInputAssemblyState: self.input_assembly.get(),
      pTessellationState: self.tesselation.get(),
      pViewportState: self.viewport.get(),
      pRasterizationState: self.raster.get(),
      pDepthStencilState: self.depth_stencil.get(),
      pMultisampleState: self.multisample.get(),
      pColorBlendState: self.blend.get(),
      pDynamicState: if self.dynamic.is_empty() {
        std::ptr::null()
      } else {
        self.dynamic.get()
      },
      layout: layout,
      renderPass: self.pass,
      subpass: self.subpass,
      basePipelineHandle: vk::NULL_HANDLE,
      basePipelineIndex: -1,
    };

    let mut handle = vk::NULL_HANDLE;
    vk::CreateGraphicsPipelines(
      self.device.get_handle(),
      vk::NULL_HANDLE,
      1,
      &create_info,
      std::ptr::null(),
      &mut handle,
    );

    stages
      .iter()
      .for_each(|s| vk::DestroyShaderModule(self.device.get_handle(), s.module, std::ptr::null()));

    Ok(Pipeline {
      device: self.device.get_handle(),
      handle,
      dsets,
      layout,
    })
  }
}
