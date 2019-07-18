use crate::rect::Rect;
use std::collections::HashMap;
use vk::builder::Buildable;
use vk::cmd::commands::BindDset;
use vk::cmd::commands::BindPipeline;
use vk::cmd::commands::DrawManaged;
use vk::cmd::commands::DrawVertices;
use vk::cmd::stream::*;
use vk::mem::Handle;
use vk::pass::DrawMeshRef;
use vk::pass::DrawPass;
use vk::pass::Framebuffer;
use vk::pass::Renderpass;

#[derive(Clone, Copy)]
enum SelectType {
  Rect(Rect),
  Empty,
}

#[derive(Clone, Copy)]
struct SelectItem {
  mesh: usize,
  typ: SelectType,
}

pub struct Select {
  rp: Renderpass,
  fb: Framebuffer,
  pass: DrawPass,
  mem: vk::mem::Mem,

  meshes: HashMap<usize, SelectItem>,

  rects_dirty: bool,
  rects_buffer: vk::Buffer,
  rects: Vec<Rect>,
}

impl Select {
  pub fn new(device: vk::Device, extent: vk::Extent2D, mem: vk::mem::Mem) -> Self {
    let mut mem = mem.clone();
    let rp = vk::pass::Renderpass::build(device)
      .attachment(
        0,
        vk::AttachmentDescription::build()
          .format(vk::FORMAT_R32_UINT)
          .initial_layout(vk::IMAGE_LAYOUT_COLOR_ATTACHMENT_OPTIMAL),
      )
      .subpass(
        0,
        vk::SubpassDescription::build().bindpoint(vk::PIPELINE_BIND_POINT_GRAPHICS).color(0),
      )
      .dependency(vk::SubpassDependency::build().external(0))
      .create()
      .unwrap();

    let fb = vk::pass::Framebuffer::build_from_pass(&rp, &mut mem.alloc).extent(extent).create();

    Self {
      rp,
      fb,
      pass: DrawPass::new(),
      mem,

      meshes: Default::default(),

      rects_dirty: false,
      rects_buffer: vk::NULL_HANDLE,
      rects: Default::default(),
    }
  }

  pub fn resize(&mut self, size: vk::Extent2D) {
    self.mem.alloc.destroy(Handle::Image(self.fb.images[0]));
    self.fb = vk::pass::Framebuffer::build_from_pass(&self.rp, &mut self.mem.alloc)
      .extent(size)
      .create();
  }

  pub fn get<'a>(&'a self, mesh: usize) -> DrawMeshRef<'a> {
    self.pass.get(mesh)
  }

  pub fn remove(&mut self, mesh: usize) {
    if let Some(s) = self.meshes.remove(&mesh) {
      let rects = &mut self.rects;
      self.pass.remove(s.mesh);
      match s.typ {
        SelectType::Rect(rect) => {
          self
            .meshes
            .iter_mut()
            .filter_map(|(mesh, s)| if let SelectType::Rect(_) = s.typ { Some(s) } else { None })
            .enumerate()
            .for_each(|(i, s)| {
              s.mesh = i;
              rects[i] = rect;
            });
          self.rects_dirty = true;
        }
        SelectType::Empty => (),
      }
    }
  }

  pub fn make_rect(&mut self, mesh: usize, rect: Rect) {
    let mut select = *self.meshes.entry(mesh).or_insert_with(|| SelectItem {
      mesh: 0,
      typ: SelectType::Empty,
    });

    match select.typ {
      SelectType::Rect(_) => (),
      _ => self.remove(mesh),
    }

    select.typ = SelectType::Rect(rect);

    select.mesh = 123;

    self.meshes.entry(mesh).and_modify(|v| *v = select);
  }

  pub fn begin(&self, cs: CmdBuffer) -> CmdBuffer {
    cs.push(&vk::cmd::commands::ImageBarrier::to_color_attachment(self.fb.images[0]))
      .push(&self.fb.begin())
      .push(&vk::cmd::commands::Viewport::with_extent(self.fb.extent))
      .push(&vk::cmd::commands::Scissor::with_extent(self.fb.extent))
  }
  pub fn end(&self, cs: CmdBuffer) -> CmdBuffer {
    cs.push(&self.fb.end())
  }
}
