use super::*;

macro_rules! make_num {
  ($name:ident, $t:ty) => {
    #[derive(Clone)]
    pub struct $name {}

    impl Parsable for $name {
      fn parse(&self, s: &str) -> Option<Vec<String>> {
        s.parse::<$t>().ok().map(|_| vec![s.into()])
      }

      fn complete(&self, s: &str) -> Option<Vec<Completion>> {
        s.parse::<$t>().ok().map(|_| vec![Completion::new(0, s.to_string())])
      }
    }

    impl $name {
      pub fn new() -> Self {
        Self {}
      }

      pub fn convert<'a>(&'a self, s: &str) -> Option<$t> {
        s.parse::<$t>().ok()
      }
    }
  };
}

make_num!(F32, f32);
make_num!(F64, f64);
make_num!(I32, i32);
make_num!(I64, i64);
make_num!(U32, u32);
make_num!(U64, u64);
make_num!(USize, usize);
make_num!(ISize, isize);
