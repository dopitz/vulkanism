#[derive(Default, Debug, Clone)]
pub struct Usings {
  pub vk: Option<String>,
  pub pipes: Option<String>,
}

impl Usings {
  pub fn get_vk(&self) -> String {
    match &self.vk {
      Some(s) => s.clone(),
      None => "vk".to_string(),
    }
  }
  pub fn get_pipes(&self) -> String {
    match &self.pipes {
      Some(s) => s.clone(),
      None => "vk::pipes".to_string(),
    }
  }
}
