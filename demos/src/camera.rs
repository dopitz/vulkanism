use assets;
use vk;
use vk::cmd::commands::BufferCopy;

pub struct Camera {
  mem: vk::mem::Mem,
  pub stage: vk::mem::Staging,
  pub ub: vk::Buffer,
  pub transform: CameraUb,
  dirty: bool,
  move_dir: vkm::Vec3i,
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct CameraUb {
  pub view: vkm::Mat4f,
  pub proj: vkm::Mat4f,
}

impl Drop for Camera {
  fn drop(&mut self) {
    self.mem.trash.push(self.ub);
  }
}

impl Camera {
  pub fn new(mut mem: vk::mem::Mem) -> Self {
    let mut ub = vk::NULL_HANDLE;
    vk::mem::Buffer::new(&mut ub)
      .uniform_buffer(std::mem::size_of::<CameraUb>() as vk::DeviceSize)
      .bind(&mut mem.alloc, vk::mem::BindType::Block)
      .unwrap();

    let stage = vk::mem::Staging::new(mem.clone(), std::mem::size_of::<CameraUb>() as vk::DeviceSize).unwrap();

    Self {
      mem,
      stage,
      ub,
      transform: Default::default(),
      dirty: true,
      move_dir: vec3!(0),
    }
  }

  pub fn handle_events(&mut self, e: &vk::winit::Event) {
    match e {
      vk::winit::Event::DeviceEvent {
        event: vk::winit::DeviceEvent::Key(vk::winit::KeyboardInput {
          state, virtual_keycode, ..
        }),
        ..
      } => {
        let dir = match state {
          vk::winit::ElementState::Pressed => 1,
          vk::winit::ElementState::Released => -1,
        };
        match virtual_keycode {
          Some(vk::winit::VirtualKeyCode::L) => self.move_dir.x += dir,
          Some(vk::winit::VirtualKeyCode::H) => self.move_dir.x += -dir,
          Some(vk::winit::VirtualKeyCode::K) => self.move_dir.z += dir,
          Some(vk::winit::VirtualKeyCode::J) => self.move_dir.z += -dir,
          _ => (),
        }
      }

      vk::winit::Event::DeviceEvent {
        event: vk::winit::DeviceEvent::MouseWheel {
          delta: vk::winit::MouseScrollDelta::LineDelta(x, y),
        },
        ..
      } => self.move_dir.y -= *y as i32,

      _ => (),
    }
  }

  pub fn resize(&mut self, fbsize: vk::Extent2D) {
    self.transform.proj = vkm::Mat4::scale(vkm::Vec3::new(1.0, -1.0, 1.0))
      * vkm::Mat4::perspective_lh(std::f32::consts::PI / 4.0, fbsize.width as f32 / fbsize.height as f32, 1.0, 100.0);
    self.dirty = true;
  }

  pub fn force_update(&mut self) {
    self.dirty = true;
  }

  pub fn update(&mut self, up: &mut assets::Update) {
    if self.move_dir != vec3!(0) {
      self.transform.view = self.transform.view * vkm::Mat4::translate(-self.move_dir.into());
      self.move_dir.y = 0;
      self.dirty = true;
    }

    if self.dirty {
      self.dirty = false;
      let mut map = self.stage.map().unwrap();
      let svb = map.as_slice_mut::<CameraUb>();
      svb[0] = self.transform;
      up.push_buffer((
        self.stage.copy_into_buffer(self.ub, 0),
        Some(vk::cmd::commands::BufferBarrier::new(self.ub).to(vk::ACCESS_UNIFORM_READ_BIT)),
      ));
    }
  }
}
