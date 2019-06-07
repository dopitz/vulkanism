mod drawpass;
mod frame;
mod pass;

pub use frame::Frame;
pub use pass::Pass;
pub use pass::PassId;
pub use pass::PassMask;

//use vk::cmd::Stream;
//
//
//pub struct DrawMeshRef<'a> {
//  pub pipe: vk::cmd::commands::BindPipeline,
//  pub dset: (usize, usize)
//  pub vb: (usize, usize),
//  pub ib: usize,
//  pub cmd: usize,
//  pub draw: vk::cmd::commands::Draw,
//}
//
//pub struct DrawMesh {
//  pub pipe: vk::cmd::commands::BindPipeline,
//  pub dset: (usize, usize)
//  pub vb: (usize, usize),
//  pub ib: usize,
//  pub cmd: usize,
//  pub draw: vk::cmd::commands::Draw,
//}
//
//
//pub struct MeshDesc {
//  pub vb: (usize, usize),
//  pub ib: usize,
//  pub cmd: usize,
//  pub dsets: (usize, usize),
//  pub pipes: usize,
//}
//
//pub struct MeshBuilder {
//  vb: Vec<vk::Buffer>,
//  ib: Vec<vk::Buffer>,
//  cmd: Vec<vk::Buffer>,
//}
//
//impl Meshes {
//  pub fn add(&mut self, buffer_count: usize, dset_count: usize, pipe_count: usize) -> (usize, usize, usize) {
//
//  }
//}
