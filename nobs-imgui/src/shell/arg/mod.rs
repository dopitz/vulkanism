mod bool;
mod file;
mod ident;

use std::ops::Range;

#[derive(Debug)]
pub struct Completion {
  score: i32,
  preview: String,
  completed: String,
  prefix: Option<String>,
  suffix: Option<String>,
}

impl Completion {
  pub fn new(score: i32, completed: String) -> Self {
    Self {
      score,
      preview: completed.clone(),
      completed,
      prefix: None,
      suffix: None,
    }
  }

  pub fn preview(mut self, pv: String) -> Self {
    self.preview = pv;
    self
  }
  pub fn prefix(mut self, pf: String) -> Self {
    self.prefix = self
      .prefix
      .map(|mut s| {
        s.push_str(&pf);
        s
      })
      .or(Some(pf));
    self
  }
  pub fn suffix(mut self, mut sf: String) -> Self {
    self.suffix = self
      .suffix
      .map(|mut s| {
        s.insert_str(0, &sf);
        s
      })
      .or(Some(sf));
    self
  }

  pub fn get_completed(&self) -> String {
    // TODO replace spaces with "\ "
    // only when required (nested call of get_complete in command...)
    format!(
      "{}{}{}",
      match self.prefix.as_ref() {
        Some(pf) => &pf,
        None => "",
      },
      self.completed,
      match self.suffix.as_ref() {
        Some(sf) => &sf,
        None => "",
      },
    )
  }

  pub fn map_completed<F: Fn(String) -> String>(mut self, f: F) -> Self {
    self.completed = f(self.completed);
    self
  }

  pub fn get_preview(&self) -> &str {
    &self.preview
  }
}

impl PartialEq for Completion {
  fn eq(&self, other: &Self) -> bool {
    self.score == other.score && self.completed == other.completed
  }
}

impl Eq for Completion {}

impl PartialOrd for Completion {
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    Some(self.cmp(other))
  }
}

impl Ord for Completion {
  fn cmp(&self, other: &Self) -> std::cmp::Ordering {
    match self.score.cmp(&other.score) {
      std::cmp::Ordering::Equal => self.completed.cmp(&other.completed),
      x => x,
    }
  }
}

pub trait Parsable {
  fn parse(&self, s: &str) -> Option<Vec<String>>;
  fn complete(&self, s: &str) -> Option<Vec<Completion>>;
}

pub use self::bool::Bool;
pub use file::File;
pub use ident::Ident;
