use vk;

pub struct ClearValueBuilder {
  clear: vk::ClearValue,
}

vk_builder_into!(vk::ClearValue, ClearValueBuilder, clear);

impl Default for ClearValueBuilder {
  fn default() -> Self {
    Self {
      clear: vk::ClearValue {
        color: vk::ClearColorValue {
          float32: [0.0, 0.0, 0.0, 0.0],
        },
      },
    }
  }
}

impl ClearValueBuilder {
  pub fn colorf32(self, c: [f32; 4]) -> Self {
    Self {
      clear: vk::ClearValue {
        color: vk::ClearColorValue { float32: c },
      },
    }
  }
  pub fn colori32(self, c: [i32; 4]) -> Self {
    Self {
      clear: vk::ClearValue {
        color: vk::ClearColorValue { int32: c },
      },
    }
  }
  pub fn coloru32(self, c: [u32; 4]) -> Self {
    Self {
      clear: vk::ClearValue {
        color: vk::ClearColorValue { uint32: c },
      },
    }
  }

  pub fn depth(self, depth: f32) -> Self {
    Self {
      clear: vk::ClearValue {
        depthStencil: vk::ClearDepthStencilValue { depth, stencil: 0 },
      },
    }
  }
  pub fn depth_stencil(self, depth: f32, stencil: u32) -> Self {
    Self {
      clear: vk::ClearValue {
        depthStencil: vk::ClearDepthStencilValue { depth, stencil },
      },
    }
  }
}
