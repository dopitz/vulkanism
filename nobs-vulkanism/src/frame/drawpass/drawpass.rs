use super::meshes::*;
use super::super::Pass;

pub struct DrawPass<T: MeshId> {
  meshes: Meshes<T>,
}

impl<T: MeshId> Pass for DrawPass<T> {
  fn run(&mut self, cmds: vk::cmd::Pool, batch: &mut vk::cmd::Frame) {
    let mut cs = cmds.begin_stream().unwrap();
    //for d in self.meshes.filter_pass() {
    //  //cs = cs.push(&d.pipe);
    //  //for ds in d.dset.iter() {
    //  //  cs = cs.push(ds);
    //  //}
    //  //cs = cs.push(&d.draw);
    //}
    batch.push(cs);
  }

  fn resize(mut self, size: vk::Extent2D) -> Self {
    self
  }
}
