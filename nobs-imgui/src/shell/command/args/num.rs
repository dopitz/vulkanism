use crate::shell::command::args::Arg;
use crate::shell::command::args::ArgDesc;
use crate::shell::command::args::Convert;
use crate::shell::command::args::Ident;
use crate::shell::command::args::Matches;

macro_rules! make_num {
  ($name:ident, $t:ty) => {
    #[derive(Clone)]
    pub struct $name {
      desc: ArgDesc,
      mi: Option<$t>,
      ma: Option<$t>,
      def: Option<$t>,
    }

    impl Arg for $name {
      fn get_desc<'a>(&'a self) -> &'a ArgDesc {
        &self.desc
      }
    }

    impl $name {
      pub fn new(desc: ArgDesc) -> Self {
        Self {
          desc,
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
      fn from_match(&self, matches: &Matches) -> Option<$t> {
        matches.value_of(&self.get_desc().name).and_then(|s| {
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
        })
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
