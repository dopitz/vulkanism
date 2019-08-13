mod pipeline;

use super::ComponentStyle;
use super::Style;
use crate::rect::Rect;
use crate::select::SelectId;
use crate::ImGui;
use std::sync::Arc;
use std::sync::Mutex;
use vk::cmd::commands::DrawManaged;
use vk::cmd::commands::DrawVertices;
use vk::pass::MeshId;

pub use pipeline::Pipeline;

#[derive(Clone)]
pub struct Simple {
  pipe: Arc<Mutex<Pipeline>>,
  ds_viewport: vk::DescriptorSet,
}

impl Style for Simple {
  type Component = ComponentSimple;

  fn new(mem: vk::mem::Mem, pass: vk::RenderPass, ds_viewport: vk::DescriptorSet) -> Self {
    let pipe = Arc::new(Mutex::new(Pipeline::new(mem.alloc.get_device(), pass, 0)));
    Self { pipe, ds_viewport }
  }
}

pub struct ComponentSimple {
  gui: ImGui<Simple>,
  ub: vk::Buffer,

  rect: Rect,
  draw_mesh: MeshId,
  select_mesh: MeshId,
  select_id_body: SelectId,
  select_id_border: SelectId,
}

impl Drop for ComponentSimple {
  fn drop(&mut self) {
    self.gui.get_mem().trash.push_buffer(self.ub);
    self.gui.get_drawpass().remove(self.draw_mesh);
    self.gui.select.remove_mesh(self.select_mesh);
    self.gui.select.remove_ids(self.select_id_body, 2);
  }
}

impl ComponentStyle<Simple> for ComponentSimple {
  fn new(gui: &ImGui<Simple>) -> Self {
    let mut mem = gui.get_mem();
    let mut ub = vk::NULL_HANDLE;
    vk::mem::Buffer::new(&mut ub)
      .uniform_buffer(std::mem::size_of::<pipeline::UbStyle>() as vk::DeviceSize)
      .devicelocal(false)
      .bind(&mut mem.alloc, vk::mem::BindType::Block)
      .unwrap();

    let pipe = Pipeline::new_color(&mut gui.style.pipe.lock().unwrap(), gui.style.ds_viewport);
    pipe.update_dsets(gui.get_device(), ub);

    let draw_mesh = gui.get_drawpass().new_mesh(
      pipe.bind_pipe,
      &[pipe.bind_ds_viewport, pipe.bind_ds_style],
      DrawManaged::new([].iter().into(), DrawVertices::with_vertices(12).into()),
    );

    let select_mesh = gui.select.new_mesh(
      pipe.bind_pipe,
      &[pipe.bind_ds_viewport, pipe.bind_ds_style],
      DrawManaged::new([].iter().into(), DrawVertices::with_vertices(12).into()),
    );
    let select_id_body = gui.select.new_ids(2);
    let select_id_border = select_id_body + 1;

    Self {
      gui: gui.clone(),
      ub,

      rect: Rect::from_rect(0, 0, 0, 0),
      draw_mesh,
      select_mesh,
      select_id_body,
      select_id_border,
    }
  }
}

impl Component<Simple> for ComponentSimple {
  fn rect(&mut self, rect: Rect) -> &mut Self {
    if self.rect != rect {
      self.rect = rect;
      let mut map = self.gui.get_mem().alloc.get_mapped(vk::mem::Handle::Buffer(self.ub)).unwrap();
      let data = map.as_slice_mut::<pipeline::UbStyle>();
      data[0].position = rect.position;
      data[0].size = rect.size.into();
      data[0].bd_thickness = vec2!(10);
    }
    self
  }
  fn get_rect(&self) -> Rect {
    self.rect
  }

  fn get_size_hint(&self) -> vkm::Vec2u {
    self.rect.size
  }

  type Event = ();
  fn draw<L: Layout>(&mut self, wnd: &mut Window<L, Simple>, _focus: &mut SelectId) -> Option<()> {
    let scissor = vk::cmd::commands::Scissor::with_rect(self.get_rect().into());
    wnd.push_draw(self.draw_mesh, scissor);

    None
  }
}

make_style!(Simple);
