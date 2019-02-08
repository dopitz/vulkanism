from collections import namedtuple

import typeid as tid


EnMember = namedtuple('EnMember', 'name value type')
En = namedtuple('En', 'intt member')


def format_enum_value(val):
    val = val.strip("()")
    suf = ""
    if "-" in val:
        s = val.split("-")
        if len(s) == 2 and len(s[0]) > 0:
            val = s[0]
            suf = " - " + s[1]
    
    elif "+" in val:
        s = val.split("-")
        val = s[0]
        suf = " + " + s[1]
    
    ty = "u32"
    if "." in val:
        ty = "f64"
    if val.endswith("f"):
        ty = "f32"
        val = val[:-1]
    if val.endswith("ULL"):
        ty = "u64"
        val = val[:-3]
    if val.endswith("U"):
        val = val[:-1]
    
    
    if val[0] == "~":
        val = "!" + val[1:]
    if val[0] == "-":
        suf = suf + "i32 as " + ty
    else:
        suf = suf + ty
    
    return (ty, val + suf)

def format_enum_bitpos(val):
    return ("u32", hex(1 << int(val)) + "u32")

def format_enum_extoffset(ext, offset, dir):
    return dir + str(1000000000 + 1000 * (int(ext) - 1) + int(offset))

def format_enum_name(name):
    return name[3:]




def parse_enum(t):
    if tid.isalias(t): return tid.makealias(t)
    return tid.maketype(t.attrib["name"], "enum", En("u32", []), [])

def parse_bitmask(t):
    if tid.isalias(t): return tid.makealias(t)

    requires = [t.find("type").text]
    if "requires" in t.attrib:
        requires.append(t.attrib["requires"])

    return tid.maketype(t.find("name").text, "bitmask", En(t.find("type").text, []), requires)

def parse_enum_member(en):
    member = []
    for e in en.findall("./enum"):
        if "value" in e.attrib:
            v = format_enum_value(e.attrib["value"])
            member.append(EnMember(e.attrib["name"], v[1], v[0]))
        if "bitpos" in e.attrib:
            v = format_enum_bitpos(e.attrib["bitpos"])
            member.append(EnMember(e.attrib["name"], v[1], v[0]))
    return member

def parse_enums(root, types):
    for en in root.findall("./enums"):
        name = en.attrib["name"]
        if name in types.types:
            t = types.types[name]
            e = t.type
            types.types[name] = tid.Type(name, t.category, En(t.type.intt, parse_enum_member(en)), t.requires)

def parse_api_constants(root):
    name = "API Constants"
    for en in root.findall("./enums"):
        if en.attrib["name"] == name:
            return tid.Type(name, "enum", En("", parse_enum_member(en)), [])

def parse_extend_enum(e, types, extnumber):
    if not "extends" in e.attrib: return
    name = e.attrib["extends"]
    if not name in types.types: return
    t = types.types[name]

    if e.attrib["name"] in map(lambda m: m.name, t.type.member): return

    if "bitpos" in e.attrib:
        v = format_enum_bitpos(e.attrib["bitpos"])
        t.type.member.append(EnMember(e.attrib["name"], v[1], v[0]))
    if "offset" in e.attrib:
        if "extnumber" in e.attrib: extnumber = e.attrib["extnumber"]
        d = ""
        if "dir" in e.attrib: d = e.attrib["dir"]
        v = format_enum_value(format_enum_extoffset(extnumber, e.attrib["offset"], d))
        t.type.member.append(EnMember(e.attrib["name"], v[1], v[0]))

        types.types[name] = tid.Type(t.name, t.category, t.type, t.requires)



def write_enum(types, t):
    s = ""
    if t.type.intt:
        s = "#[doc(hidden)] pub type " + types.format_type(t.name) + " = " + types.format_type(t.type.intt) + ";\n"
    for e in t.type.member:
        s += "#[doc(hidden)] pub const " + format_enum_name(e.name) + ": " + e.type + " = " + e.value + ";\n";
    return s


def write_result(types):
    member = types.types["VkResult"].type.member

    s = "\n\n\n"
    s += "/// Wraps a call to a vulkan command and converts it's returned error code with [make_result](fn.make_result.html).\n"
    s += "///\n"
    s += "/// Prints debug information to std::out with the file and line number of the vulkan command.\n"
    s += "/// This macro may only be used with vulkan commands that return a `nobs_vk::Result`\n"
    s += "///\n"
    s += "/// ## Example\n"
    s += "/// ```rust\n"
    s += "/// #[macro_use] extern crate nobs_vk as vk;\n"
    s += "/// //...\n"
    s += "/// # fn main() {\n"
    s += "/// # let _vk_lib = vk::Core::new();\n"
    s += "/// # let mut inst_ver = 0;\n"
    s += "/// match vk_check!(vk::EnumerateInstanceVersion(&mut inst_ver)) {\n"
    s += "///   Err(e) => println!(\"EnumerateInstanceVersion returned with: {:?}\", e),\n"
    s += "///   Ok(e) => println!(\"EnumerateInstanceVersion returned with: {:?}\", e),\n"
    s += "/// }\n"
    s += "/// # }\n"
    s += "/// ```\n"
    s += "#[macro_export]\n"
    s += "macro_rules! vk_check {\n"
    s += "  ($fn:expr) => {{\n"
    s += "    let r = $crate::make_result($fn);\n"
    s += "    if let Err(e) = r {\n"
    s += "      println!(\n"
    s += "        \"{} failed with {:?} in \\\"{}\\\" at line {}\",\n"
    s += "        stringify!($fn),\n"
    s += "        e,\n"
    s += "        file!(),\n"
    s += "        line!()\n"
    s += "      );\n"
    s += "    }\n"
    s += "    r\n"
    s += "  }};\n"
    s += "}\n"
    s += "\n"
    s += "/// Same as [vk_check](macro.vk_check.html) but instead of returning the Result calls unwrap on it.\n"
    s += "///\n"
    s += "/// Still prints debug information to std::out with the file and line number of the vulkan command.\n"
    s += "#[macro_export]\n"
    s += "macro_rules! vk_uncheck {\n"
    s += "  ($fn:expr) => {{ vk_check!($fn).unwrap() }};\n"
    s += "}\n"
    s += "\n"
    s += "\n"
    s += "/// Converts the integer error code from a vulkan command into a `Result<Success, Error>`\n"
    s += "pub fn make_result(r: crate::Result) -> std::result::Result<Success, Error> {\n"
    s += "  match r {\n"

    for m in member:
        name = format_enum_name(m.name)
        if m.value.startswith('-'):
            s += "    " + name + " => Err(Error::" + name + "),\n"
        else:
            s += "    " + name + " => Ok(Success::" + name + "),\n"

    s += "    _ => Err(Error::UNKNOWN)\n"
    s += "  }\n"
    s += "}\n"
    s += "\n"
    s += "/// Enum type for all successful return codes in `nobs_vk::Result`\n"
    s += "#[derive(Debug, Clone, Copy)]\n"
    s += "pub enum Success {\n"
    for m in member:
        if not m.value.startswith('-'):
            s += "  " + format_enum_name(m.name) + ",\n"
    s += "}\n"
    s += "impl Success {\n"
    s += "  /// Gets the actual vulkan return code back.\n"
    s += "  pub fn vk_result(&self) -> crate::Result {\n"
    s += "    match self {\n"
    for m in member:
        if not m.value.startswith('-'):
            name = format_enum_name(m.name)
            s += "      Success::" + name + " => crate::" + name + ",\n"
    s += "    }\n"
    s += "  }\n"
    s += "}\n"
    s += "\n"

    s += "/// Enum type for all unsuccessful return codes in `nobs_vk::Result`\n"
    s += "#[derive(Debug, Clone, Copy)]\n"
    s += "pub enum Error  {\n"
    for m in member:
        name = format_enum_name(m.name)
        if m.value.startswith('-'):
            s += "  " + format_enum_name(m.name) + ",\n"
    s += "  UNKNOWN,\n"
    s += "}\n"
    s += "impl Error {\n"
    s += "  /// Gets the actual vulkan return code back.\n"
    s += "  pub fn vk_result(&self) -> crate::Result {\n"
    s += "    match self {\n"
    for m in member:
        if m.value.startswith('-'):
            name = format_enum_name(m.name)
            s += "      Error::" + name + " => crate::" + name + ",\n"
    s += "      Error::UNKNOWN => crate::Result::max_value(),\n"
    s += "    }\n"
    s += "  }\n"
    s += "}\n"
    s += "\n"
    return s


