use vk::cmd::commands::BindDset;
use vk::cmd::commands::BindPipeline;
use vk::cmd::commands::Draw;

pub struct DrawMeshRef<'a> {
  pub pipe: BindPipeline,
  pub dset: &'a [BindDset],
  pub vb: &'a [vk::Buffer],
  pub ib: vk::Buffer,
  pub cmd: vk::Buffer,
  //pub draw: Draw,
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
  type Pass;
  fn filter(&self, p: Self::Pass) -> bool;
}

pub struct Meshes<T: MeshId> {
  buffers: Vec<vk::Buffer>,
  dsets: Vec<BindDset>,
  pipes: Vec<BindPipeline>,

  meshes: Vec<(T, DrawMesh)>,
}

impl<T: MeshId> Meshes<T> {
  fn make_ref<'a>(&'a self, draw: &DrawMesh) -> DrawMeshRef<'a> {
    DrawMeshRef {
      pipe: self.pipes[draw.pipe],
      dset: &self.dsets[draw.dset.0..draw.dset.1],
      vb: &self.buffers[draw.vb.0..draw.vb.1],
      ib: self.buffers[draw.ib],
      cmd: self.buffers[draw.cmd],
      //draw: draw.draw,
    }
  }

  pub fn filter_pass<'a>(&'a self, pass: T::Pass) -> impl Iterator<Item = DrawMeshRef<'a>> {
    self
      .meshes
      .iter()
      .filter_map(|(id, draw)| if id.filter(pass) { Some(self.make_ref(draw)) } else { None })
  }
}
