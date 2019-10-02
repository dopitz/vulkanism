use super::*;

macro_rules! make_num {
  ($name:ident, $t:ty) => {
    #[derive(Clone)]
    pub struct $name {
      mi: Option<$t>,
      ma: Option<$t>,
      def: Option<$t>,
    }

    impl Parsable for $name {
      fn can_parse(&self, s: &str) -> bool {
        self.convert(s).is_some()
      }

      fn complete(&self, s: &str) -> Option<Vec<Completion>> {
        s.parse::<$t>().ok().map(|_| vec![Completion::new(0, s.to_string())])
      }
    }

    impl $name {
      pub fn new() -> Self {
        Self {
          mi: None,
          ma: None,
          def: None,
        }
      }

      pub fn min(mut self, v: $t) -> Self {
        self.mi = Some(v);
        self
      }
      pub fn max(mut self, v: $t) -> Self {
        self.ma = Some(v);
        self
      }
      pub fn default(mut self, v: $t) -> Self {
        self.def = Some(v);
        self
      }
    }

    impl Convert<$t> for $name {
      fn convert<'a>(&'a self, s: &str) -> Option<$t> {
        s.parse::<$t>()
          .ok()
          .filter(|v| match self.mi {
            Some(mi) => mi <= *v,
            None => true,
          })
          .filter(|v| match self.ma {
            Some(ma) => ma > *v,
            None => true,
          })
      }
    }

    impl ConvertDefault<$t> for $name {
      fn default(&self) -> Option<$t> {
        self.def
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
