use crate::cmd::CmdBuffer;
use super::StreamPush;
use vk;

/// Issues a dispatch call for compute pipelines in a command stream
pub enum Dispatch {
  Base(DispatchBase),
  Indirect(DispatchIndirect),
}
pub struct DispatchBase {
  pub x: u32,
  pub y: u32,
  pub z: u32,
}
pub struct DispatchIndirect {
  pub offset: vk::DeviceSize,
  pub buffer: vk::Buffer,
}

impl Dispatch {
  pub fn x(x: u32) -> Self {
    Self::xyz(x, 1, 1)
  }
  pub fn xy(x: u32, y: u32) -> Self {
    Self::xyz(x, y, 1)
  }
  pub fn xyz(x: u32, y: u32, z: u32) -> Self {
    Dispatch::Base(DispatchBase { x, y: y, z: z })
  }

  pub fn indirect(buffer: vk::Buffer) -> Self {
    Self::indirect_offset(buffer, 0)
  }
  pub fn indirect_offset(buffer: vk::Buffer, offset: vk::DeviceSize) -> Self {
    Dispatch::Indirect(DispatchIndirect { offset, buffer })
  }
}

impl StreamPush for Dispatch {
  fn enqueue(&self, cs: CmdBuffer) -> CmdBuffer {
    match self {
      Dispatch::Base(d) => vk::CmdDispatch(cs.buffer, d.x, d.y, d.z),
      Dispatch::Indirect(d) => vk::CmdDispatchIndirect(cs.buffer, d.buffer, d.offset),
    }
    cs
  }
}
