use proc_macro::TokenStream;
use proc_macro::TokenTree;

use std::io::prelude::*;
use std::process::Command;

use binding::Binding;
use parse;
use shader::stage_from_stirng;
use shader::Builder as ShaderBuilder;
use shader::Shader;
use std::collections::HashMap;
use usings::Usings;

#[derive(Default, Debug)]
pub struct Builder {
  pub usings: Usings,
  pub stages: Vec<ShaderBuilder>,
  pub dset_names: HashMap<u32, String>,
  pub dump: String,
}

const ARG_TYPES: &[&str] = &["dset_name[N]", "dump", "inlude", "stage"];

impl Builder {
  pub fn from_tokens(input: TokenStream) -> Result<Builder, String> {
    let mut b = Builder::default();
    let mut includes = Vec::new();
    let mut tokens = input.clone().into_iter();
    while let Some(tok) = tokens.next() {
      match tok {
        TokenTree::Ident(i) => {
          let s = i.to_string();

          let idx = match s.as_ref() {
            "dset_name" => parse::parse_array_index(&mut tokens).map_err(|e| format!("after argument dset_name: {}", e))?,
            _ => 0,
          };

          tokens.next();
          match s.as_ref() {
            "nobs_vkpipes_alias" => b.usings.pipes = Some(parse::parse_string(&mut tokens)),
            "nobs_vk_alias" => b.usings.vk = Some(parse::parse_string(&mut tokens)),
            "stage" => b.stages.push(parse::parse_group(&mut tokens, ShaderBuilder::from_tokens)?),
            "include" => includes = parse::parse_string_vec(&mut tokens).map_err(|e| format!("after argument include: {}", e))?,
            "dset_name" => {
              b.dset_names.insert(idx, parse::parse_string(&mut tokens));
            }
            "dump" => b.dump = parse::parse_string(&mut tokens),
            _ => Err(format!("expected one of {:?}, found {}", ARG_TYPES, &s))?,
          }
        }
        TokenTree::Group(_) => Err("expected TokenTree::Ident, found TokenTree::Group")?,
        TokenTree::Literal(_) => Err("expected TokenTree::Ident, found TokenTree::Literal")?,
        TokenTree::Punct(_) => Err("expected TokenTree::Ident, found TokenTree::Punct")?,
      }
    }

    for i in includes.iter() {
      for stage in b.stages.iter_mut() {
        if !stage.includes.iter().any(|inc| inc == i) {
          stage.includes.push(i.to_string());
        }
      }
    }

    if b.stages.is_empty() {
      Err("no shader stage was specified")?;
    }

    if b.stages.iter().any(|b| b.stage == "comp") && b.stages.len() > 1 {
      Err("compute pipelines must only contain a single compute stage and can not be mixed with shaders of different stage types")?;
    }

    for s in b.stages.iter_mut() {
      s.usings = b.usings.clone();
    }

    Ok(b)
  }

  pub fn new(&self) -> Result<Pipeline, String> {
    let mut stages = Vec::with_capacity(self.stages.len());
    for b in self.stages.iter() {
      stages.push(b.new()?);
    }

    let mut bindings: Vec<Binding> = Vec::new();
    for s in stages.iter() {
      let stage_bit = stage_from_stirng(&s.stage).unwrap();
      for b in s.bindings.iter() {
        // if the binding id is different we also require a different name
        if bindings.iter().any(|bind| bind.binding == b.binding && bind.name != b.name)
          || bindings.iter().any(|bind| bind.binding != b.binding && bind.name == b.name)
        {
          Err(format!("binding name collision for binding {:?}", b))?;
        }

        // if the binding already exists add the shader stage to binding
        // else we create a new binding for the pipeline
        if let Some(p) = bindings.iter().position(|bind| Binding::same_stage(bind, b)) {
          bindings[p].stageflags |= stage_bit;
        } else {
          bindings.push(b.clone());
        }
      }
    }

    let mut dset_names = bindings
      .iter()
      .fold(std::collections::HashSet::new(), |mut acc, b| {
        acc.insert(b.descset);
        acc
      })
      .iter()
      .fold(HashMap::new(), |mut acc, s| {
        match self.dset_names.get(s) {
          Some(v) => acc.entry(*s).or_insert(v.clone()),
          None => acc.entry(*s).or_insert(format!("dset{}", s)),
        };
        acc
      });

    if dset_names.len() == 1 && self.dset_names.is_empty() {
      dset_names.values_mut().for_each(|v| *v = "dset".to_string());
    }

    Ok(Pipeline {
      usings: self.usings.clone(),
      stages,
      dset_names,
      bindings,
    })
  }
}

pub struct Pipeline {
  pub usings: Usings,
  pub stages: Vec<Shader>,
  pub dset_names: HashMap<u32, String>,
  pub bindings: Vec<Binding>,
}

impl Pipeline {
  fn write_bindings(&self, bindings: &[Binding]) -> String {
    format!(
      "
        pub const BINDINGS: [Binding; {}] = [
          {}
        ];
        ",
      bindings.len(),
      bindings
        .iter()
        .fold(String::new(), |acc, b| format!("{}{}", acc, b.to_binding_string()))
    )
  }

  fn write_descriptors(&self, descriptors: &HashMap<u32, Vec<Binding>>) -> String {
    descriptors.iter().fold(String::new(), |acc, (set, b)| {
      format!(
        "
        {acc}\n
        pub mod {name} {{
          use {vk_alias};
          use {vkpipes_alias};

          use {vkpipes_alias}::pipeline::Binding;
          use {vkpipes_alias}::descriptor::writes::Writes;
          use {vk_alias}::DescriptorBufferInfo;
          use {vk_alias}::DescriptorImageInfo;
          use {vk_alias}::BufferView;

          {bindings}

          pub fn write(device: vk::Device, dset: vk::DescriptorSet) -> Write {{
            Write::new(device, dset)
          }}

          pub struct Write {{
            inner: Writes,
          }}

          impl Write {{
            pub fn new(device: vk::Device, dset: vk::DescriptorSet) -> Write {{
              Write {{
                inner: Writes::new(device, dset),
              }}
            }}

            {setter}

            pub fn update(&mut self) {{
              self.inner.update();
            }}
          }}
        }}
        ",
        acc = acc,
        vk_alias = self.usings.get_vk(),
        vkpipes_alias = self.usings.get_pipes(),
        bindings = self.write_bindings(b),
        name = self.dset_names[set],
        setter = b
          .iter()
          .fold(String::new(), |acc, b| format!("{}{}", acc, b.to_dset_write_string("Write"))),
      )
    })
  }

  fn write_compute(&self) -> String {
    "pub fn new(device: vk::Device) -> BuilderComp {{
        let mut b = BuilderComp::from_device(device);
        b.bindings(&BINDINGS).comp(&comp::create_module(device));
        b
      }}"
      .to_string()
  }

  fn write_graphics(&self) -> String {
    format!(
      "pub fn new(device: vk::Device, pass: vk::RenderPass, subpass: u32) -> BuilderGraphics {{
        let mut b = BuilderGraphics::from_pass(device, pass, subpass);
        b.bindings(&BINDINGS)
        {stages};
        b
      }}",
      stages = self.stages.iter().fold(String::new(), |acc, s| format!(
        "{acc}.{name}(&{name}::create_module(device))\n",
        acc = acc,
        name = s.stage
      ))
    )
  }

  pub fn write_module(&self) -> String {
    let stages = self.stages.iter().fold(String::new(), |acc, s| {
      format!(
        "{acc}\npub mod {stage} {{ {shader} }}",
        acc = acc,
        stage = s.stage,
        shader = s.write_module()
      )
    });

    let descriptors = self.bindings.iter().fold(std::collections::HashMap::new(), |mut acc, b| {
      {
        let bs = acc.entry(b.descset).or_insert(Vec::new());
        bs.push(b.clone());
      }
      acc
    });

    let build = match self.stages.iter().find(|s| s.stage == "comp") {
      Some(_) => self.write_compute(),
      None => self.write_graphics(),
    };

    format!(
      "
      use {vk_alias};

      use {vkpipes_alias}::pipeline::Binding;
      use {vkpipes_alias}::descriptor::writes::*;
      use {vkpipes_alias}::pipeline::builder::compute::Compute as BuilderComp;
      use {vkpipes_alias}::pipeline::builder::graphics::Graphics as BuilderGraphics;

      use std::collections::HashMap;

      {stages}

      {bindings}

      pub const NUM_SETS: u32 = {num_sets};

      {build}

      {descriptors}
      ",
      vk_alias = self.usings.get_vk(),
      vkpipes_alias = self.usings.get_pipes(),
      bindings = self.write_bindings(&self.bindings),
      build = build,
      num_sets = descriptors.len(),
      stages = stages,
      descriptors = self.write_descriptors(&descriptors),
    )
  }

  pub fn dump(&self, filename: &str) -> Result<(), String> {
    let mut f = std::fs::File::create(&filename).map_err(|_| format!("Could not open file: {}", filename))?;
    f.write_all(self.write_module().as_bytes())
      .map_err(|_| "Could not write pipeline module to file")?;

    Command::new("sh")
      .arg("-c")
      .arg(format!("rustfmt {}", filename))
      .output()
      .map_err(|_| "Could not format pipeline module file, is rustfmt installed?")?;
    Ok(())
  }
}
