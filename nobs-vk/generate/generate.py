import xml.etree.ElementTree as ET
from os.path import realpath, dirname

import basetype
import handle
import enum
import struct
import typeid
import commands
import funcptr
import features
import exttypes


tree = ET.parse(dirname(realpath(__file__))+"/vk.xml")
root = tree.getroot()


types = typeid.Types(root)

enum.parse_enums(root, types)
apiconst = enum.parse_api_constants(root)
struct.parse_impldbg(types)

cmds = commands.Commands(root)



features.require_feature(root, "VK_VERSION_1_0", types, cmds)
features.require_feature(root, "VK_VERSION_1_1", types, cmds)

exceptext = []

extnames = typeid.Type("EXTENSION_NAMES", "enum", enum.En("", []), [])
for ext in root.findall("./extensions/extension"):
    if features.is_empty_extension(ext): continue
    if ext.attrib["name"] in exceptext: continue
    features.require_extension(root, ext.attrib["name"], types, cmds, extnames)



f = open(dirname(realpath(__file__)) + "/../src/lib.rs", "w")

def write_crate_doc():
    s = []
    s.append("# nobs-vk\n")
    s.append("no bullshit vulkan bindings.\n")
    s.append("\n")
    s.append("This crate is auto generated by python scripts and provides types, constants and functions for [vulkan](https://www.khronos.org/vulkan/).\n")
    s.append("\n")
    s.append("1. [Existential questions](#existential-questions)\n")
    s.append("2. [Examples](#examples)\n")
    s.append("    1. [Vulkan core initialisation](#vulkan-core-initialisation)\n")
    s.append("    2. [Convenience Instance and Device creation](#convenience-instance-and-device-creation)\n")
    s.append("3. [Details](#details)\n")
    s.append("    1. [Namespaces](#namespaces)\n")
    s.append("    2. [Function pointers](#function-pointers)\n")
    s.append("    3. [Check macros](#check-macros)\n")
    s.append("    4. [Instance and Device builder patterns](#instance-and-device-builder-patterns)\n")
    s.append("\n")
    s.append("## Existential questions\n")
    s.append("Why does nobs-vk exists? nobs-vk...\n")
    s.append("1. is used how the vulkan api is documented\n")
    s.append("2. is auto generated from a python script that sources the vk.xml from the vulkan registry\n")
    s.append("3. gives you the freedom to do as you please in your design decisions (but therefore doesn't protect you from your own stupidity)\n")
    s.append("4. is not a full blown window creating bloat library in the back, just to execute some small compute shader with a headless vulkan build\n")
    s.append("\n")
    s.append("While more involved wrappers for vulkan do exist they also strife to completely hide the vulkan api behind another layer of rust code and might force you into design decisions you would normally try to avoid. This library tries to be as simple as possible and just exposes callable functions to vulkan.\n")
    s.append("\n")
    s.append("## Examples\n")
    s.append("### Vulkan core initialisation\n")
    s.append("This is a simple example that retrieves the version of vulkan that is installed on the system\n")
    s.append("```rust\n")
    s.append("#[macro_use] extern crate nobs_vk as vk;\n")
    s.append("//...\n")
    s.append("\n")
    s.append("# fn main() {\n")
    s.append("// loads vulkan\n")
    s.append("let _vk_lib = vk::VkLib::new();\n")
    s.append("\n")
    s.append("// good to go from here, we can use any vulkan function that is supported by this system\n")
    s.append("// make sure `_vk_lib` lives throughout the time that vulkan is used and is dropped afterwards\n")
    s.append("\n")
    s.append("// global vulkan version\n")
    s.append("let mut inst_ver: u32 = 0;\n")
    s.append("if vk::EnumerateInstanceVersion(&mut inst_ver) != vk::SUCCESS { \n")
    s.append("  panic!(\"something went terribly wrong\");\n")
    s.append("}\n")
    s.append("\n")
    s.append("assert_eq!(1, version_major!(inst_ver));\n")
    s.append("assert_eq!(1, version_minor!(inst_ver));\n")
    s.append("assert_eq!(0, version_patch!(inst_ver));\n")
    s.append("# }\n")
    s.append("```\n")
    s.append("\n")
    s.append("### Convenience Instance and Device creation\n")
    s.append("Instance and device creation are a large portion of the boiler plate code that comes with implementing a vulkan application, so it makes sence to have a convenient way of doing this in the library (which is why you could argue that it barely does not contradicts the \"no bullshit\" paradigm)\n")
    s.append("```rust\n")
    s.append("#[macro_use]\n")
    s.append("extern crate nobs_vk;\n")
    s.append("\n")
    s.append("use nobs_vk as vk;\n")
    s.append("use std::ffi::CStr;\n")
    s.append("\n")
    s.append("fn main() {\n")
    s.append("  let lib = vk::VkLib::new();\n")
    s.append("  let inst = vk::instance::new()\n")
    s.append("    .validate(vk::DEBUG_REPORT_ERROR_BIT_EXT | vk::DEBUG_REPORT_WARNING_BIT_EXT)\n")
    s.append("    .application(\"awesome app\", 0)\n")
    s.append("    .add_extension(vk::KHR_SURFACE_EXTENSION_NAME)\n")
    s.append("    .add_extension(vk::KHR_XLIB_SURFACE_EXTENSION_NAME)\n")
    s.append("    .create(lib)\n")
    s.append("    .unwrap();\n")
    s.append("\n")
    s.append("  for pd in vk::device::PhysicalDevice::enumerate_all(inst.handle) {\n")
    s.append("    println!(\n")
    s.append("      \"instance api version:  {} {} {}\",\n")
    s.append("      version_major!(pd.properties.apiVersion),\n")
    s.append("      version_minor!(pd.properties.apiVersion),\n")
    s.append("      version_patch!(pd.properties.apiVersion)\n")
    s.append("    );\n")
    s.append("    println!(\"driver version:        {}\", pd.properties.driverVersion);\n")
    s.append("    println!(\"vendor id:             {}\", pd.properties.vendorID);\n")
    s.append("    println!(\"device id:             {}\", pd.properties.deviceID);\n")
    s.append("    println!(\"vendor:                {}\", unsafe {\n")
    s.append("      CStr::from_ptr(&pd.properties.deviceName[0]).to_str().unwrap()\n")
    s.append("    });\n")
    s.append("    \n")
    s.append("    println!(\"layers:                {:?}\", pd.supported_layers);\n")
    s.append("    println!(\"extensions:            {:?}\", pd.supported_extensions);\n")
    s.append("  }\n")
    s.append("\n")
    s.append("  let (_pdevice, _device) = vk::device::PhysicalDevice::enumerate_all(inst.handle)\n")
    s.append("    .remove(0)\n")
    s.append("    .into_device()\n")
    s.append("    .add_queue(vk::device::QueueProperties {\n")
    s.append("      present: false,\n")
    s.append("      graphics: true,\n")
    s.append("      compute: true,\n")
    s.append("      transfer: true,\n")
    s.append("    }).create()\n")
    s.append("    .unwrap();\n")
    s.append("}\n")
    s.append("```\n")
    s.append("\n")
    s.append("## Details\n")
    s.append("### Namespaces\n")
    s.append("Name prefixes of the C-types, enums and functions are removed in favor of the module namespace. \n")
    s.append("For example `VK_Result` becomes `vk::Result`, `VK_SUCCESS` becomes `vk::SUCCESS`, `vkCreateInstance()` becomes `vk::CreateInstance()`.\n")
    s.append("\n")
    s.append("### Function pointers\n")
    s.append("Entry points to vulkan commands are stored in [VkLib](struct.VkLib.html). There are also functions declared globally for every vulkan command. After creating an instance of `VkLib` these function redirect to the `VkLib` instance. This is done for convenience purposes, so that we do not have to pass on the `VkLib` instance. Since there is a function for every vulkan command, this also includes commands, that are not supported on the system. In this case calling the function will panic even after instance/device creation. The same will happen if the vkulan library was initialized with a feature level and the function is therefore not supported.\n")
    s.append("\n")
    s.append("### Check macros\n")
    s.append("Additionally to the [result integer constants](https://www.khronos.org/registry/vulkan/specs/1.1-extensions/man/html/VkResult.html) that are defined by the vulkan api, the two enums [Success](enum.Success.html) and [Error](enum.Error.html) are declared. These capture the successful and unsuccessful error codes. The [vk_check!](macro.vk_check.html) converts the error code returned from vulkan with [make_result](fn.make_result.html) and prints debug information when the command failed. `vk_uncheck!` will consume the result and panic on error, while `vk_check!` returns the `Result<Success, Error>`\n")
    s.append("\n")
    s.append("### Instance and Device builder patterns\n")
    s.append("As the sole convenience feature this library introduces builder patterns for instance and device creation. This enables a convenient way of configuring e.g. debug layers for a vulkan instance, or extensions and properties of queues for devices. See [instance::Builder](instance/struct.Builder.html) and [device::Builder](device/struct.Builder.html) for more details\n")
    s.append("\n")
    s.append("## Vulkan reference\n")
    s.append("For documentation of the defined enums, structs and funcions see the \n")
    s.append("[vulkan reference](https://www.khronos.org/registry/vulkan/specs/1.1-extensions/man/html/).\n")
    return "//! " + "//! ".join(s)
f.write(write_crate_doc() + "\n\n")

f.write("#![allow(non_upper_case_globals)]\n")
f.write("#![allow(non_snake_case)]\n")
f.write("#![allow(non_camel_case_types)]\n")
f.write("\n")
f.write("use std::os::raw::c_char;\n")
f.write("use std::os::raw::c_ushort;\n")
f.write("use std::os::raw::c_ulong;\n")
f.write("use std::os::raw::c_void;\n")
f.write("use std::mem;\n")
f.write("\n")

f.write("\n")
f.write("\n")

for t in types.get_types(["basetype"], True):
    f.write(basetype.write_basetype(types, t))

f.write("\n")
f.write("\n")
f.write(enum.write_enum(types, apiconst))

f.write("\n")
f.write("\n")
f.write(enum.write_enum(types, extnames))

f.write("\n")
f.write("\n")

f.write(handle.write_null_handle())
for t in types.get_types(["handle"], True):
    f.write(handle.write_handle(types, t))

f.write("\n")
f.write("\n")

for t in types.get_types(["funcpointer"], True):
    f.write(funcptr.write_ptr(t) + "\n")

f.write("\n")
f.write("\n")

for t in types.get_types(["bitmask", "enum"], True):
    f.write(enum.write_enum(types, t) + "\n")

f.write("\n")
f.write("\n")

for t in types.get_types(["struct", "union"], True):
    f.write(struct.write_struct(types, t))

f.write("\n")
f.write("\n")

for t in types.get_types(["exttype"], True):
    f.write(exttypes.write_type(types, t))

f.write("\n")
f.write("\n")

f.write(cmds.write(types))

f.write(enum.write_result(types))

f.write("\n\n")
f.write("pub mod device;\n")
f.write("pub mod instance;\n")
f.write("pub mod builder;\n")

f.write("\n")
f.close()



