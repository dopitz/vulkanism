use proc_macro::TokenStream;
use proc_macro::TokenTree;

use std::io::prelude::*;
use std::path::Path;
use std::process::Command;

use binding::Binding;
use parse;
use spirv;
use usings::Usings;
use vk;

#[derive(Default, Debug)]
pub struct Builder {
  pub usings: Usings,
  pub stage: String,
  pub entry: String,
  pub path_glsl: String,
  pub path_spv: String,
  pub src_glsl: String,
  pub src_spv: Vec<u32>,
  pub includes: Vec<String>,
  pub dump: String,
}

const ARG_TYPES: &[&str] = &[
  "nobs_vkpipes_alias",
  "nobs_vk_alias",
  "ty",
  "entry",
  "glsl",
  "spv",
  "include",
  "dump",
];
const STAGE_TYPES: &[&str] = &["vert", "tesc", "tese", "geom", "frag", "comp"];

pub fn stage_from_stirng(stage: &str) -> Result<vk::ShaderStageFlagBits, String> {
  match stage.as_ref() {
    "vert" => Ok(vk::SHADER_STAGE_VERTEX_BIT),
    "tesc" => Ok(vk::SHADER_STAGE_TESSELLATION_CONTROL_BIT),
    "tese" => Ok(vk::SHADER_STAGE_TESSELLATION_EVALUATION_BIT),
    "geom" => Ok(vk::SHADER_STAGE_GEOMETRY_BIT),
    "frag" => Ok(vk::SHADER_STAGE_FRAGMENT_BIT),
    "comp" => Ok(vk::SHADER_STAGE_COMPUTE_BIT),
    _ => Err(format!("shader stage \"{}\" not recognized, use one of {:?}", stage, STAGE_TYPES))?,
  }
}

impl Builder {
  pub fn from_tokens(input: TokenStream) -> Result<Builder, String> {
    let mut b = Builder::default();
    let mut tokens = input.clone().into_iter();
    while let Some(tok) = tokens.next() {
      match tok {
        TokenTree::Ident(i) => {
          tokens.next();
          let s = i.to_string();
          match s.as_ref() {
            "nobs_vkpipes_alias" => b.usings.pipes = Some(parse::parse_string(&mut tokens)),
            "nobs_vk_alias" => b.usings.vk = Some(parse::parse_string(&mut tokens)),
            "ty" => b.stage = parse::parse_string(&mut tokens),
            "entry" => b.entry = parse::parse_string(&mut tokens),
            "glsl" => b.path_glsl = parse::parse_string(&mut tokens),
            "spv" => b.path_spv = parse::parse_string(&mut tokens),
            "include" => b.includes = parse::parse_string_vec(&mut tokens).map_err(|e| format!("after argument include: {}", e))?,
            "dump" => b.dump = parse::parse_string(&mut tokens),
            _ => Err(format!("expected one of {:?}, found {}", ARG_TYPES, &s))?,
          }
        }
        TokenTree::Group(_) => Err("expected TokenTree::Ident, found TokenTree::Group")?,
        TokenTree::Literal(_) => Err("expected TokenTree::Ident, found TokenTree::Literal")?,
        TokenTree::Punct(_) => Err("expected TokenTree::Ident, found TokenTree::Punct")?,
      }
    }

    if !b.path_glsl.is_empty() && !b.src_spv.is_empty() {
      Err("Both \"glsl\" and \"spv\" have been specified for shader source")?
    }

    // first try to read the src from a spcified spv file
    if !b.path_spv.is_empty() {
      let path = Self::make_abs_path(&b.path_spv).ok_or(format!("Could not find {}", b.path_spv))?;
      let mut f = std::fs::File::open(&path).map_err(|_| format!("Could not open {}", b.path_spv))?;
      let mut buf = Vec::new();
      f.read_to_end(&mut buf).map_err(|_| format!("Could not read {}", b.path_spv))?;
      unsafe {
        b.src_spv.resize(buf.len() / 4, 0);
        std::ptr::copy_nonoverlapping(std::mem::transmute(buf.as_ptr()), b.src_spv.as_mut_ptr(), buf.len() / 4);
      }
    }

    // if that failed, we try to read the file as glsl source (either load from file, inline source)
    if !b.path_glsl.is_empty() {
      let path = Self::make_abs_path(&b.path_glsl).ok_or(format!("Could not find {}", b.path_glsl))?;
      if std::path::Path::new(&path).is_file() {
        let mut f = std::fs::File::open(&path).map_err(|_| format!("Could not open {}", b.path_glsl))?;
        f.read_to_string(&mut b.src_glsl)
          .map_err(|_| format!("Could not read {}", b.path_glsl))?;
      } else {
        b.src_glsl = b.path_glsl;
        b.path_glsl = String::new();
      }
    }

    // At this point we got to have either of those specified
    if b.src_spv.is_empty() && b.src_glsl.is_empty() {
      Err("Neither \"glsl\" nor \"spv\" have been specified for shader source")?
    }

    // our entry point by default is main
    if b.entry.is_empty() {
      b.entry = "main".to_string();
    }

    // add the parent folder of the root glsl source file to the include paths (if glsl is specified)
    if !b.path_glsl.is_empty() {
      if let Some(parent) = Path::new(&b.path_glsl).parent() {
        b.includes.push(parent.to_str().unwrap().to_owned());
      }
    }

    stage_from_stirng(&b.stage)?;

    Ok(b)
  }

  pub fn new(&self) -> Result<Shader, String> {
    // make include dirs absolute
    let includes = self.includes.iter().filter_map(|i| Self::make_abs_path(i)).collect::<Vec<_>>();

    // if we have glsl source we comile it with shaderc
    // if not we use the spv from the file
    let binary = if !self.src_glsl.is_empty() {
      let mut compiler = shaderc::Compiler::new().unwrap();
      let mut options = shaderc::CompileOptions::new().unwrap();

      options.set_include_callback(|includer, include_type, includee, depth| {
        Self::find_include(includer, include_type, includee, depth, &includes, !self.path_glsl.is_empty())
      });

      let shader_kind = match self.stage.as_ref() {
        "vert" => shaderc::ShaderKind::Vertex,
        "tesc" => shaderc::ShaderKind::TessControl,
        "tese" => shaderc::ShaderKind::TessEvaluation,
        "geom" => shaderc::ShaderKind::Geometry,
        "frag" => shaderc::ShaderKind::Fragment,
        "comp" => shaderc::ShaderKind::Compute,
        _ => Err(format!(
          "shader stage \"{}\" not recognized, use one of {:?}",
          self.stage, STAGE_TYPES
        ))?,
      };

      let asm = compiler
        .compile_into_spirv(
          self.src_glsl.as_ref(),
          shader_kind,
          self.path_glsl.as_str(),
          &self.entry,
          Some(&options),
        )
        .map_err(|e| format!("shader compilation failed:\n {}", e))?;

      asm.as_binary().to_vec()
    } else {
      self.src_spv.clone()
    };

    // parse the spirv to get uniforms
    let spirv = spirv::Spirv::from_binary(&binary)?;

    Ok(Shader {
      usings: self.usings.clone(),
      stage: self.stage.clone(),
      entry: self.entry.clone(),
      bindings: spirv.get_bindings(stage_from_stirng(&self.stage)?),
      binary: binary,
    })
  }

  fn make_abs_path(path: &String) -> Option<String> {
    let p = std::path::Path::new(path);
    if p.exists() {
      Some(path.clone())
    } else {
      let abs = std::env::var("CARGO_MANIFEST_DIR").unwrap().to_owned() + "/" + path;
      let p = std::path::Path::new(&abs);
      if p.exists() {
        Some(abs.clone())
      } else {
        None
      }
    }
  }

  fn find_include(
    includee: &str,
    include_type: shaderc::IncludeType,
    includer: &str,
    depth: usize,
    include_dirs: &[String],
    root_source_has_path: bool,
  ) -> Result<shaderc::ResolvedInclude, String> {
    let resolved = match include_type {
      shaderc::IncludeType::Relative => {
        if !root_source_has_path && depth == 1 {
          return Err("Can not find includes with relative path to the root source file, when using embedded GLSL.".to_string());
        }

        let parent_dir = Path::new(includer).parent().unwrap();
        let resolved = parent_dir.join(includee);

        if !resolved.is_file() {
          return Err(format!("Include `{}` is not a file, included from `{}`", includee, includer));
        }
        resolved
      }

      shaderc::IncludeType::Standard => {
        let includee_path = Path::new(includee);

        if includee_path.is_absolute() {
          return Err(format!("Include `{}` is not a file, included from `{}`", includee, includer));
        }

        let mut found = None;

        for dir in include_dirs {
          let path = Path::new(dir)
            .canonicalize()
            .unwrap_or_else(|_| panic!("`{}` is not a valid include directory.", dir));

          if let Ok(resolved) = path.join(includee).canonicalize() {
            if resolved.is_file() {
              found = Some(resolved);
              break;
            }
          }
        }

        if found.is_none() {
          return Err(format!("Include `{}` is not a file, included from `{}`", includee, includer));
        }

        found.unwrap()
      }
    }
    .canonicalize()
    .unwrap();


    let resolved_name = resolved.to_str().unwrap().to_string();

    let content = {
      let mut buffer = String::new();
      let mut f = std::fs::File::open(&resolved_name).unwrap();
      f.read_to_string(&mut buffer).unwrap();
      buffer
    };

    Ok(shaderc::ResolvedInclude { resolved_name, content })
  }
}

pub struct Shader {
  pub usings: Usings,
  pub stage: String,
  pub entry: String,
  pub bindings: Vec<Binding>,
  pub binary: Vec<u32>,
}

impl Shader {
  fn write_bindings(&self) -> String {
    format!(
      "
      pub const BINDINGS: [Binding; {}] = [
        {}
      ];
      ",
      self.bindings.len(),
      self
        .bindings
        .iter()
        .fold(String::new(), |acc, b| format!("{}{}", acc, b.to_binding_string()))
    )
  }

  pub fn write_module(&self) -> String {
    format!(
      "
      use {vk_alias};
      use {vkpipes_alias}::pipeline::Binding;

      const SPIRV: &[u32] = &[
        {spirv}
      ];

      pub fn create_module(device: vk::Device) -> vk::PipelineShaderStageCreateInfo {{
        let create_info = vk::ShaderModuleCreateInfo {{
          sType: vk::STRUCTURE_TYPE_SHADER_MODULE_CREATE_INFO,
          pNext: std::ptr::null(),
          flags: 0,
          codeSize: SPIRV.len() * std::mem::size_of::<u32>(),
          pCode: SPIRV.as_ptr(),
        }};
  
        let mut module = vk::NULL_HANDLE;
        vk::CreateShaderModule(device, &create_info, std::ptr::null(), &mut module);
        vk::PipelineShaderStageCreateInfo {{
          sType: vk::STRUCTURE_TYPE_PIPELINE_SHADER_STAGE_CREATE_INFO,
          pNext: std::ptr::null(),
          flags: 0,
          stage: {shader_stage},
          module: module,
          pName: b\"{shader_entry}\\0\".as_ptr() as *const std::os::raw::c_char,
          pSpecializationInfo: std::ptr::null(),
        }}
        
      }}

      {bindings}
      ",
      vk_alias = self.usings.get_vk(),
      vkpipes_alias = self.usings.get_pipes(),
      shader_stage = stage_from_stirng(&self.stage).unwrap(),
      shader_entry = self.entry,
      spirv = self.binary.iter().fold(String::new(), |s, w| format!("{} {},", s, w)),
      bindings = self.write_bindings(),
    )
  }

  pub fn dump(&self, filename: &str) -> Result<(), String> {
    let mut f = std::fs::File::create(&filename).map_err(|_| format!("Could not open file: {}", filename))?;
    f.write_all(self.write_module().as_bytes())
      .map_err(|_| "Could not write shader module to file")?;

    Command::new("sh")
      .arg("-c")
      .arg(format!("rustfmt {}", filename))
      .output()
      .map_err(|_| "Could not format shader module file, is rustfmt installed?")?;
    Ok(())
  }
}
