use vk::cmd::commands::BindDset;
use vk::cmd::commands::BindPipeline;
use vk::cmd::commands::Draw;
use std::collections::HashMap;

pub struct DrawMeshRef<'a> {
  pub pipe: BindPipeline,
  pub dset: &'a [BindDset],
  pub vb: &'a [vk::Buffer],
  pub ib: vk::Buffer,
  pub cmd: vk::Buffer,
  pub draw: &'a Draw,
}

struct DrawMesh {
  pub pipe: usize,
  pub dset: (usize, usize),
  pub vb: (usize, usize),
  pub ib: usize,
  pub cmd: usize,
  pub draw: Draw,
}

pub trait MeshId {
  type Pass: std::hash::Hash + PartialEq + Eq + Clone + Copy;
  fn filter(&self, p: Self::Pass) -> bool;
}

pub struct Meshes<T: MeshId> {
  buffers: Vec<vk::Buffer>,
  dsets: Vec<BindDset>,
  pipes: Vec<BindPipeline>,

  meshes: Vec<(T, DrawMesh)>,
}

impl<T: MeshId> Meshes<T> {
  fn make_ref<'a>(&'a self, draw: &'a DrawMesh) -> DrawMeshRef<'a> {
    DrawMeshRef {
      pipe: self.pipes[draw.pipe],
      dset: &self.dsets[draw.dset.0..draw.dset.1],
      vb: &self.buffers[draw.vb.0..draw.vb.1],
      ib: self.buffers[draw.ib],
      cmd: self.buffers[draw.cmd],
      draw: &draw.draw,
    }
  }

  pub fn filter_pass<'a>(&'a self, pass: T::Pass) -> impl Iterator<Item = DrawMeshRef<'a>> {
    self
      .meshes
      .iter()
      .filter_map(move |(id, draw)| if id.filter(pass) { Some(self.make_ref(draw)) } else { None })
  }
}


struct MeshBuilderPass {
  pipe: BindPipeline,
  dset: Vec<BindDset>,
  vb: Vec<vk::Buffer>,
  ib: Option<vk::Buffer>,
  cmd: Option<vk::Buffer>,
  draw: Draw,
}

pub struct MeshBuilder<T: MeshId> {
  passes: HashMap<T::Pass, MeshBuilderPass>,
}

impl<T: MeshId> MeshBuilder<T> {
  pub fn add(self, meshes: &mut Meshes<T>) {
    
  //  let mut draw = DrawMesh {
  //pub pipe: usize,
  //pub dset: (usize, usize),
  //pub vb: (usize, usize),
  //pub ib: usize,
  //pub cmd: usize,
  //pub draw: Draw,

  //  }

  }
}
