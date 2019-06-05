use vk;

pub trait Pass {
  fn run(&mut self, cmds: vk::cmd::Pool, batch: &mut vk::cmd::Frame);

  fn resize(mut self, size: vk::Extent2D) -> Self;
}

