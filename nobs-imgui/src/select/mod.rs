pub mod rects;
mod selectpass;

pub use rects::Rects as SelectRects;
pub use selectpass::Query;
pub use selectpass::SelectPass;
pub use selectpass::SelectId;

use crate::pipeid::*;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::MutexGuard;
use vk::cmd::commands::BindDset;
use vk::cmd::commands::BindPipeline;
use vk::cmd::commands::DrawKind;
use vk::cmd::commands::DrawManaged;

/// Wraps a [SelectPass](struct.SelectPass.html) and other useful object selection managers
///
/// Declares the same interface as [SelectPass](struct.SelectPass.html).
///
/// Sets up managers for
///  - [Rects](rects/struct.Rects.html) - Manages selectable objects on rectangular regions of the screen
#[derive(Clone)]
pub struct Select {
  pass: SelectPass,
  rects: Arc<Mutex<rects::Rects>>,
}

impl Select {
  /// Creates a new [SelectPass](struct.SelectPass.html) and managers for selectable objects.
  pub fn new(pass: SelectPass, pipes: &PipeCache, mem: vk::mem::Mem) -> Self {
    let rects = Arc::new(Mutex::new(SelectRects::new(pass.clone(), &pipes, mem)));
    Self { pass, rects }
  }

  /// Resize the [SelectPass](struct.SelectPass.html)
  ///
  /// See [resize](struct.SelectPass.html#method.resize).
  pub fn resize(&mut self, size: vk::Extent2D) {
    self.pass.resize(size);
  }

  /// Resize the [SelectPass](struct.SelectPass.html)
  ///
  /// See [handle_events](struct.SelectPass.html#method.handle_events).
  pub fn handle_events(&mut self, e: &vk::winit::Event) {
    self.pass.handle_events(e);
  }

  /// Resize the [SelectPass](struct.SelectPass.html)
  ///
  /// See [new_id](struct.SelectPass.html#method.new_id).
  pub fn new_id(&mut self) -> SelectId {
    self.pass.new_id()
  }
  /// Resize the [SelectPass](struct.SelectPass.html)
  ///
  /// See [new_ids](struct.SelectPass.html#method.new_ids).
  pub fn new_ids(&mut self, count: u32) -> SelectId {
    self.pass.new_ids(count)
  }
  /// Resize the [SelectPass](struct.SelectPass.html)
  ///
  /// See [remove_id](struct.SelectPass.html#method.remove_id).
  pub fn remove_id(&mut self, id: SelectId) {
    self.pass.remove_id(id);
  }
  /// Resize the [SelectPass](struct.SelectPass.html)
  ///
  /// See [remove_ids](struct.SelectPass.html#method.remove_ids).
  pub fn remove_ids(&mut self, id: SelectId, count: u32) {
    self.pass.remove_ids(id, count);
  }

  /// Resize the [SelectPass](struct.SelectPass.html)
  ///
  /// See [new_mesh](struct.SelectPass.html#method.new_mesh).
  pub fn new_mesh(&mut self, pipe: BindPipeline, dsets: &[BindDset], draw: DrawManaged) -> usize {
    self.pass.new_mesh(pipe, dsets, draw)
  }

  /// Resize the [SelectPass](struct.SelectPass.html)
  ///
  /// See [update_mesh](struct.SelectPass.html#method.update_mesh).
  pub fn update_mesh(
    &mut self,
    mesh: usize,
    pipe: Option<BindPipeline>,
    dsets: &[Option<BindDset>],
    buffers: &[Option<vk::Buffer>],
    draw: Option<DrawKind>,
  ) {
    self.pass.update_mesh(mesh, pipe, dsets, buffers, draw);
  }

  /// Resize the [SelectPass](struct.SelectPass.html)
  ///
  /// See [contains](struct.SelectPass.html#method.contains).
  pub fn contains(&self, mesh: usize) -> bool {
    self.pass.contains(mesh)
  }

  /// Resize the [SelectPass](struct.SelectPass.html)
  ///
  /// See [remove_mesh](struct.SelectPass.html#method.remove_mesh).
  pub fn remove_mesh(&mut self, mesh: usize) -> bool {
    self.pass.remove_mesh(mesh)
  }

  /// Resize the [SelectPass](struct.SelectPass.html)
  ///
  /// See [push_query](struct.SelectPass.html#method.push_query).
  pub fn push_query<'a>(&self, q: &'a mut Query) -> selectpass::PushQuery<'a> {
    self.pass.push_query(q)
  }

  /// Resize the [SelectPass](struct.SelectPass.html)
  ///
  /// See [get_renderpass](struct.SelectPass.html#method.get_renderpass).
  pub fn get_renderpass(&self) -> vk::RenderPass {
    self.pass.get_renderpass()
  }

  /// Gets the manager for [Rects](rects/struct.Rects.html)
  pub fn rects<'a>(&'a self) -> MutexGuard<'a, SelectRects> {
    self.rects.lock().unwrap()
  }
}
