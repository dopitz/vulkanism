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
  Custom,
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
  rects_buffer_len: usize,
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
      rects_buffer_len: 0,
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
    // SelectItems are refenced by outside mesh id
    if let Some(s) = self.meshes.remove(&mesh) {
      // We do not need the select mesh any more
      self.pass.remove(s.mesh);
      match s.typ {
        SelectType::Rect(rect) => {
          // pop the last element in the rect vector
          // and update all rects of the remaining SelectItems
          let rects = &mut self.rects;
          rects.pop();
          self
            .meshes
            .iter_mut()
            .filter_map(|(mesh, s)| if let SelectType::Rect(_) = s.typ { Some(s) } else { None })
            .enumerate()
            .for_each(|(i, s)| {
              // reference new position of the rect
              s.mesh = i;
              rects[i] = rect;
            });
          self.rects_dirty = true;
        }
        _ => (),
      }
    }
  }

  pub fn make_rect(&mut self, mesh: usize, rect: Rect) {
    // Copy SelectItem and write back later, because we might need to remove the mesh if it was a different type
    let mut select = *self.meshes.entry(mesh).or_insert_with(|| SelectItem {
      mesh: 0,
      typ: SelectType::Empty,
    });

    // If the type of this SelectItem was not a rect we, remove it and create a new one with correct type
    match select.typ {
      SelectType::Rect(_) => (),
      _ => {
        self.remove(mesh);
        select.mesh = self.rects.len();
        self.rects.push(rect);
      }
    }

    select.typ = SelectType::Rect(rect);

    // write back
    self.meshes.entry(mesh).and_modify(|v| *v = select);
    self.rects_dirty = true;
  }

  fn update_rects(&mut self) {
    if self.rects_dirty {
      self.rects_dirty = false;

      if self.rects.len() != self.rects_buffer_len {
        self.mem.trash.push_buffer(self.rects_buffer);
        self.rects_buffer = vk::NULL_HANDLE;

        vk::mem::Buffer::new(&mut self.rects_buffer)
          .vertex_buffer((self.rects.len() * std::mem::size_of::<Rect>()) as vk::DeviceSize)
          .devicelocal(false)
          .bind(&mut self.mem.alloc, vk::mem::BindType::Block)
          .unwrap();

        self.rects_buffer_len = self.rects.len();
      }

      // only copy if not empty
      if !self.rects.is_empty() {
        self
          .mem
          .alloc
          .get_mapped(Handle::Buffer(self.rects_buffer))
          .unwrap()
          .host_to_device_slice(&self.rects);
      }
    }
  }

  pub fn begin(&mut self, cs: CmdBuffer) -> CmdBuffer {
    self.update_rects();

    cs.push(&vk::cmd::commands::ImageBarrier::to_color_attachment(self.fb.images[0]))
      .push(&self.fb.begin())
      .push(&vk::cmd::commands::Viewport::with_extent(self.fb.extent))
      .push(&vk::cmd::commands::Scissor::with_extent(self.fb.extent))
  }
  pub fn end(&self, cs: CmdBuffer) -> CmdBuffer {
    cs.push(&self.fb.end())
  }
}
