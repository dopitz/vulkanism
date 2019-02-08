from collections import namedtuple

import typeid as tid
import decl

Param = namedtuple('Param', 'name type')
Proto = namedtuple('Proto', 'name params ret')
Cmd = namedtuple('Cmd', 'proto requires')

class Commands:
    root = []
    commands = {}
    commandfeatures = {}
    features = []
    orderedcmds = []

    used = {}
    aliases = {}

    def __init__(self, root):
        self.root = root 

        for cs in root.findall("./commands"):
            for c in cs.findall("./command"):

                if "alias" in c.attrib:
                    self.aliases[c.attrib["name"]] = c.attrib["alias"]
                    continue

                proto = c.find("proto")
                if proto == None:
                    print "no proto"

                name = proto.find("name").text
                ret = proto.find("type").text

                requires = [ret]
                params = []
                for p in c.findall("param"):
                    d = decl.parse_decl(p)
                    params.append(d)
                    if not d.type in tid.ctypes and not d.type in requires: requires.append(d.type)

                proto = Proto(name, params, ret)
                self.commands[name] = Cmd(proto, requires)
                self.used[name] = False


    def require(self, name, feature):
        if name in self.aliases:
            name = self.aliases[name]

        if not name in self.commands:
            print "COMMAND NOT FOUND " + name
            return []

        if not self.used[name]:
            self.used[name] = True
            self.commandfeatures[name] = feature
            if not feature.version in self.features:
                self.features.append(feature.version)
                self.features.sort()
            self.orderedcmds.append(name)
            return self.commands[name].requires

        return []


    def write(self, types):
	def make_commands(structname, entry):
            def params(cmd, flags = ""):
                s = ""
                for i in range(0, len(cmd.proto.params)):
                    p = cmd.proto.params[i]
                    d = decl.resolve_decl(types, p)
                    if 'u' in flags:
                        d = decl.Decl("_" + d.name, d.type, d.decl);
                    if 'n' in flags:
                        s += d.name
                    else:
                        s += decl.write_decl(d)
                    if i < len(cmd.proto.params) - 1:
                        s += ", "
                return s

            def self_params(cmd):
                return "&self, " + params(cmd)

            def params_names(cmd):
                return params(cmd, "n")

            def params_unused(cmd):
                return params(cmd, "u")

            def name(cmd):
                return cmd.proto.name[2:]

            def name_ptr(cmd):
                return name(cmd) + "_ptr"

            def name_panic(cmd):
                return name(cmd) + "_panic"

            def ret_type(ret):
                if ret != "void":
                    return " -> " + types.format_type(ret)
                else:
                    return ""

            extern_sys = "extern \"system\" "
            def fn_proto(decl, cmd, name = lambda cmd: "", params = lambda cmd: params(cmd)):
                return decl + "fn " + name(cmd) + "(" + params(cmd) + ")" + ret_type(cmd.proto.ret)

            # function that always panics, for not loaded commands
            def fn_panic(cmd):
                s = ""
                s += fn_proto("extern \"system\" ", cmd, name_panic, params_unused) + "{\n"
                s += "    panic!(\"extension " + cmd.proto.name + " not loaded\")\n"
                s += "}\n"
                return s

            # load command
            def load_cmd(cmd):
                def transmute(exp):
                    return "mem::transmute(" + exp + ")"
                def symbol(cmd):
                    return "vk_dl.symbol::<c_void>(\"" + cmd.proto.name + "\")"
                def expect(cmd):
                    return "expect(\"could not find symbol for " + cmd.proto.name + "\")"

                version = const_version(self.commandfeatures[cmd.proto.name].version)
                proto_panic = fn_proto(extern_sys, cmd);

                if entry == "core":
                    s = ""
                    s += "let " + name_ptr(cmd) + " = if feature < " + version + " {\n"
                    s += "  " + name_panic(cmd) + " as " + proto_panic + "\n"
                    s += "} else {\n"
                    s += "  " + transmute(symbol(cmd) + "." + expect(cmd)) + "\n"
                    s += "};\n"
                    return s

                else:
                    proto_name = fn_proto(extern_sys, cmd, name_panic);

                    hname = "h" + entry;
                    procname = "Get" + entry[0].upper() + entry[1:] + "ProcAddr"

                    s = ""
                    s += "name = std::ffi::CString::new(\"" + cmd.proto.name + "\").unwrap();\n"
                    s += "let " + name_ptr(cmd) + " = if feature < " + version + " {\n"
                    s += "  " + name_panic(cmd) + " as " + proto_panic + "\n"
                    s += "} else {\n"
                    s += "  ptr = " + procname + "(" + hname + ", name.as_ptr());\n"
                    s += "  if (ptr as *const c_void).is_null() {\n"
                    s += "    " + name_panic(cmd) + " as " + proto_panic + "\n"
                    s += "  } else {\n"
                    s += "    mem::transmute(ptr)\n"
                    s += "  }\n"
                    s += "};\n\n"
                    return s

            # vulkan command as call to member of function pointer wrapper
            def call_ptr(caller, cmd):
                semi = ""
                if ret_type(cmd.proto.ret) == "()":
                    semi = ";"

                return "(" + caller + "." + name_ptr(cmd) + ")(" + params_names(cmd) + ")" + semi + "\n"

            def call_member(cmd):

                s = ""
                s += fn_proto("#[doc(hidden)] pub ", cmd, name, self_params) + "{\n"
                s += "  " + call_ptr("self", cmd)
                s += "}\n"
                return s

            # vulkan command as call to global function in the nobs_vk namespace (core commands only)
            def call_static(cmd):
                semi = ""
                if ret_type(cmd.proto.ret) == "()":
                    semi = ";"

                s = ""
                s += fn_proto("#[doc(hidden)] pub ", cmd, name) + "{\n"
                s += "  unsafe {\n"
                s += "    let ptr = core.expect(\"Vulkan core not initialized, make sure to have a valid instance of nobs_vk::Core\");\n"
                s += "    " + call_ptr("(*ptr)", cmd)
                s += "  }\n"
                s += "}\n"
                return s

            # execute lambda on every command and concat strings
            def for_each_command(doit, feature = None):
                s = ""
                for name in self.orderedcmds:
                    if not self.used[name]: continue
                    if self.commandfeatures[name].entry != entry: continue
                    s += doit(self.commands[name])
                return s

	    def indent(text, prefix):
        	def prefixed_lines():
        	    for line in text.splitlines(True):
        	        yield (prefix + line)
        	return ''.join(prefixed_lines())


            hname = "h" + entry;
            htype = entry[0].upper() + entry[1:];

            # struct definition
	    s = ""

            if entry == "core":
                s += "/// Vulkan core commands\n"
                s += "///\n"
                s += "/// This struct is used to initialize vulkan core commands and holds function pointers to them.\n"
                s += "/// Function pointers are initialized during construction.\n"
                s += "///\n"
                s += "/// After successfull instantiation the global core functions are ready to use.\n"
                s += "/// There must be always at maximum a single instance of this struct.\n"
            elif entry == "instance":
                s += "/// Vulkan instance extensions\n"
                s += "///\n"
                s += "/// This struct is used to initialize vulkan instance extensions and holds function pointers to them.\n"
                s += "/// Function pointers are initialized during construction.\n"
                s += "#[derive(Clone,Copy)]"
            elif entry == "device":
                s += "/// Vulkan device extensions\n"
                s += "///\n"
                s += "/// This struct is used to initialize vulkan device extensions and holds function pointers to them.\n"
                s += "/// Function pointers are initialized during construction.\n"
                s += "#[derive(Clone,Copy)]"

            s += "pub struct " + structname + " {\n"
            s += "  #[allow(dead_code)]\n"
            if entry == "core":
                s += "  lib: shared_library::dynamic_library::DynamicLibrary,\n"
                s += "  feature: u32,\n"
            else:
                s += "  " + hname + ": " + htype + ",\n"
            s +=      indent(for_each_command(lambda cmd: name_ptr(cmd) + ": " + fn_proto(extern_sys, cmd) + ",\n"), "  ")
            s += "}\n"

            # functions for default initialization and in case the vulkan command could not be loaded
            s += "\n"
            s += "\n"
            s += for_each_command(lambda cmd: fn_panic(cmd))

            # default trait
            s += "\n"
            s += "\n"
            s += "impl Default for " + structname + " {\n"
            s += "  /// Initializes all function pointers to functions that immediately panic.\n"
            s += "  fn default() -> " + structname + " {\n"
            s += "    " + structname + " {\n"
            if entry == "core":
                s += "      lib: shared_library::dynamic_library::DynamicLibrary::open(None).expect(\"can not open library\"),\n"
                s += "      feature: 0,\n"
            else:
                s += "      " + hname + ": NULL_HANDLE,\n"
            s +=              indent(for_each_command(lambda cmd: name_ptr(cmd) + ": " + name_panic(cmd) + ",\n"), "      ")
            s += "    }\n"
            s += "  }\n"
            s += "}\n"

            # impl new() loads commands
            s += "\n"
            s += "\n"
            s += "impl " + structname + " {\n"
            if entry == "core":
                s += "  /// Initialized core commands for the newest available vulkan version\n"
                s += "  /// \n"
                s += "  /// ```\n"
                s += "  /// let vk_lib = nobs_vk::Core::new();\n"
                s += "  /// ```\n"
                s += "  /// is the same as\n"
                s += "  /// ```\n"
                s += "  /// let vk_lib = nobs_vk::Core::with_feature(nobs_vk::VERSION_1_1);\n"
                s += "  /// ```\n"
                s += "  pub fn new() -> std::boxed::Box<" + structname + "> {\n"
                s += "    Self::with_feature(" + const_version(self.features[-1]) + ")\n"
                s += "  }\n"

                s += "  /// Initialized core commands for the specified vulkan feature\n"
                s += "  /// \n"
                s += "  /// Select a feature either with the predefined constants `VERSION_x_x`,\n"
                s += "  /// or use the [make_version](macro.make_version.html) macro\n"
                s += "  pub fn with_feature(feature: u32) -> std::boxed::Box<" + structname + "> {\n"
            else:
                s += "  /// Initialized " + entry + " extensions\n"
                s += "  /// \n"
                s += "  /// A valid instance of [Core](struct.Core.html) is needen to successfully initialize extensions.\n"
                s += "  /// The vulkan feature level is picked up through the current Core instance.\n"
                s += "  pub fn new(" + hname + ": " + htype + ") -> " + structname + " {\n"

            # core will load the dynamic_library
            # extensions will use the vkGetInstanceProcAddr and vkGetDeviceProcAddr from core
            if entry  == "core":
                s += "    #[cfg(windows)]\n"
                s += "    fn open_lib() -> shared_library::dynamic_library::DynamicLibrary {\n"
                s += "      shared_library::dynamic_library::DynamicLibrary::open(Some(std::path::Path::new(\"vulkan-1.dll\"))).expect(\"vulkan not found\");\n"
                s += "    }\n"
                s += "    #[cfg(all(unix, not(target_os = \"android\"), not(target_os = \"macos\")))]\n"
                s += "    fn open_lib() -> shared_library::dynamic_library::DynamicLibrary {\n"
                s += "      shared_library::dynamic_library::DynamicLibrary::open(Some(std::path::Path::new(\"libvulkan.so\"))).expect(\"vulkan not found\")\n"
                s += "    }\n"
                s += "    #[cfg(target_os = \"macos\")]\n"
                s += "    fn open_lib() -> shared_library::dynamic_library::DynamicLibrary {\n"
                s += "      shared_library::dynamic_library::DynamicLibrary::open(Some(std::path::Path::new(\"libvulkan.1.dylib\"))).expect(\"vulkan not found\")\n"
                s += "    }\n"
                s += "    let vk_dl = open_lib();\n"
            else:
                s += "    let mut name;\n"
                s += "    let mut ptr;\n"

            # load the commands
            s += "\n"
            s += "    unsafe { \n"
            if entry != "core":
                s += "      let feature = (*core.expect(\"Vulkan core not initialized, make sure to have a valid instance of nobs_vk::Core\")).feature;\n"
            s +=              indent(for_each_command(load_cmd), "        ")
            s += "\n"

            # core returns a box with the dynamic_library and sets the static pointer
            if entry == "core":
                s += "      let c = " + structname + " {\n"
                s += "        lib: vk_dl,\n"
                s += "        feature,\n"
                s +=          indent(for_each_command(lambda cmd: name_ptr(cmd) + ",\n"), "        ")
                s += "      };\n"
                s += "\n"
                s += "      core = Some(std::boxed::Box::into_raw(std::boxed::Box::new(c)));\n"
                s += "      std::boxed::Box::from_raw(core.unwrap())"
            else:
                s += "      " + structname + " {\n"
                s += "        " + hname + ",\n"
                s +=          indent(for_each_command(lambda cmd: name_ptr(cmd) + ",\n"), "        ")
                s += "      }\n"
            s += "    }\n"
            s += "  }\n"
            s += "\n"
            s += "\n"
            if entry == "core":
                s += "  /// Gets the feature level with which vulkan was initialized\n"
                s += "  ///\n"
                s += "  /// The fature is formatted as described in [make_version](macro.make_version.html).\n"
                s += "  pub fn get_feature(&self) -> u32 {\n"
                s += "    self.feature\n"
                s += "  }\n"
            else:
                s += "  /// Retrieve the vulkan handle of the " + structname + "\n"
                s += "  pub fn get_handle(&self) -> " + htype + " {\n"
                s += "    self." + hname + "\n"
                s += "  }\n"
            s += "\n"
            s +=      indent(for_each_command(lambda cmd: call_member(cmd)), "  ")
            s += "}\n"
            
            # additional for core 
            #   Drop, to reset the static pointer
            #   static pointer
            #   public function that call functions on static pointer
            if entry == "core":
                s += "impl Drop for " + structname + " {\n"
                s += "  fn drop(&mut self) {\n"
                s += "    unsafe {\n"
                s += "      core = None;\n"
                s += "    }\n"
                s += "  }\n"
                s += "}\n"
                s += "\n"
                s += "\n"
                s += "static mut core: Option<*mut Core> = None;\n"
                s += "\n"
                s += for_each_command(lambda cmd: call_static(cmd))

	    return s


        s = ""
        s += "/// Create a version number from a major, minor and patch as it is defined in\n"
        s += "/// [vulkan version numbers and semantics](https://www.khronos.org/registry/vulkan/specs/1.1-extensions/html/vkspec.html#fundamentals-versionnum)\n"
        s += "/// \n"
        s += "/// May be used to set the `applicationVersion`, `pEngineName` and `apiVersion` fields in [ApplicationInfo](https://www.khronos.org/registry/vulkan/specs/1.1-extensions/man/html/VkApplicationInfo.htm)\n"
        s += "#[macro_export]\n"
        s += "macro_rules! make_version {\n"
        s += "  ($major:expr, $minor:expr, $patch:expr) => ((($major) << 22) | (($minor) << 12) | ($patch))\n"
        s += "}\n"

        s += "/// Extract major number from version, created with [make_version](macro.make_version.html) or retrieved from vulkan\n"
        s += "#[macro_export]\n"
        s += "macro_rules! version_major {\n"
        s += "  ($ver:expr) => ($ver >> 22)\n"
        s += "}\n"

        s += "/// Extract minor number from version, created with [make_version](macro.make_version.html) or retrieved from vulkan\n"
        s += "#[macro_export]\n"
        s += "macro_rules! version_minor {\n"
        s += "  ($ver:expr) => (($ver >> 12) & 0x3ff)\n"
        s += "}\n"

        s += "/// Extract patch number from version, created with [make_version](macro.make_version.html) or retrieved from vulkan\n"
        s += "#[macro_export]\n"
        s += "macro_rules! version_patch {\n"
        s += "  ($ver:expr) => ($ver & 0xfff)\n"
        s += "}\n"

        def const_version(f):
            return f[3:]

        for f in self.features:
            numbers = f.replace("VK_VERSION_", "").split("_")
            s += "pub const " + f[3:] + ": u32 = make_version!(" + numbers[0] + ", " + numbers[1] + ", 0);\n"

        s += "\n"
        s += "\n"
	s += make_commands("Core", "core")
        s += "\n"
        s += "\n"
	s += make_commands("InstanceExtensions", "instance")
        s += "\n"
        s += "\n"
	s += make_commands("DeviceExtensions", "device")



        def trait_default(cmd, entry):
            def is_device_cmd():
                if self.commandfeatures[cmd.proto.name].entry == "core":
                    handles = types.categories["handle"]
                    for p in cmd.proto.params:
                        if p.type in handles: 
                            h = p.type
                            while h != "":
                                if h == "VkDevice": return True
                                h = types.types[h].type.parent
                    return False
                else:
                    return self.commandfeatures[cmd.proto.name].entry == "device"

            def params(names, vkhandlename = None):
                ps = []
                if not names:
                    ps.append("&self")
                for i in range(0, len(cmd.proto.params)):
                    p = cmd.proto.params[i]
                    d = decl.resolve_decl(types, p)
                    if names:
                        if d.type == vkhandlename:
                            ps.append("self.get_extensions().get_handle()")
                        else:
                            ps.append(d.name)
                    else:
                        if d.type != vkhandlename:
                            ps.append(decl.write_decl(d))
                return ", ".join(ps)

            def ret_type():
                if cmd.proto.ret != "void":
                    return " -> " + types.format_type(cmd.proto.ret)
                else:
                    return ""

            if cmd.proto.name in [
                    "vkGetInstanceProcAddr", "vkCreateInstance", "vkDestroyInstance", 
                    "vkGetDeviceProcAddr", "vkCreateDevice", "vkDestroyDevice"]: return ""

            device_cmd = is_device_cmd()
            core_cmd = self.commandfeatures[cmd.proto.name].entry == "core"

            semi = ""
            if cmd.proto.ret == "void":
                semi = ";"

            s = ""
            if entry == "instance" and not device_cmd:
                s += "  fn " + cmd.proto.name[2:] + "(" + params(False, "Instance") + ")" + ret_type() + " {\n"
                if core_cmd:
                    s += "    " + cmd.proto.name[2:] + "(" + params(True, "Instance") + ")" + semi + "\n"
                else:
                    s += "    self.get_extensions()." + cmd.proto.name[2:] + "(" + params(True, "Instance") + ")" + semi + "\n"
                s += "  }\n"

            if entry == "device" and device_cmd:
                s += "  fn " + cmd.proto.name[2:] + "(" + params(False, "Device") + ")" + ret_type() + " {\n"
                if core_cmd:
                    s += "    " + cmd.proto.name[2:] + "(" + params(True, "Device") + ")" + semi + "\n"
                else:
                    s += "    self.get_extensions()." + cmd.proto.name[2:] + "(" + params(True, "Device") + ")" + semi + "\n"
                s += "  }\n"

            return s




        s += "\n"
        s += "\n"
        s += "\n"
        s += "/// Trait with default pass-through implementations for vulkan commands associated with an instance\n"
        s += "/// # ATTENTION - This trait might be changed or removed in the future\n"
        s += "///\n"
        s += "/// # Details\n"
        s += "/// This trait only requires the implementation of [get_extensions](trait.InstanceWrapper.html#tymethod.get_extensions).\n"
        s += "/// Instance level vulkan commands are then passed through to their respective core or instance extension function pointer.\n"
        s += "/// If the vulkan command required an `Instance` parameter, the wrapper will drop this parameter, since the\n"
        s += "/// instance handle can be provided by the [InstanceExtensions](struct.InstanceExtensions.html).\n"
        s += "///\n"
        s += "/// # Examples\n"
        s += "///\n"
        s += "/// ```\n"
        s += "/// #[macro_use] extern crate nobs_vk as vk;\n"
        s += "/// use vk::InstanceWrapper;\n"
        s += "/// # fn main() {\n"
        s += "/// \n"
        s += "/// use std::ptr;\n"
        s += "/// \n"
        s += "/// // Simple instance to only hold the vulkan core lib and instance extensions.\n"
        s += "/// struct MyInstance {\n"
        s += "///   vklib: std::boxed::Box<vk::Core>,\n"
        s += "///   extensions: vk::InstanceExtensions,\n"
        s += "/// }\n"
        s += "/// \n"
        s += "/// impl MyInstance {\n"
        s += "///   // We can provide a convenient constructor.\n"
        s += "///   pub fn new() -> MyInstance {\n"
        s += "///     let vklib = vk::Core::new();\n"
        s += "/// \n"
        s += "///     // create an instance\n"
        s += "///     let appinfo = vk::InstanceCreateInfo {\n"
        s += "///       sType: vk::STRUCTURE_TYPE_INSTANCE_CREATE_INFO,\n"
        s += "///       pNext: ptr::null(),\n"
        s += "///       flags: 0,\n"
        s += "///       pApplicationInfo: ptr::null(),\n"
        s += "///       enabledLayerCount: 0,\n"
        s += "///       ppEnabledLayerNames: ptr::null(),\n"
        s += "///       enabledExtensionCount: 0,\n"
        s += "///       ppEnabledExtensionNames: ptr::null(),\n"
        s += "///     };\n"
        s += "/// \n"
        s += "///     let mut inst = vk::NULL_HANDLE;\n"
        s += "///     vk::CreateInstance(&appinfo, ptr::null(), &mut inst);\n"
        s += "/// \n"
        s += "///     // carefull, here we should do error checking...\n"
        s += "///     let extensions = vk::InstanceExtensions::new(inst);\n"
        s += "/// \n"
        s += "///     MyInstance { vklib, extensions }\n"
        s += "///   }\n"
        s += "/// }\n"
        s += "/// \n"
        s += "/// // Implement the wrapper trait, so that we can call vulkan commands as member functions\n"
        s += "/// impl vk::InstanceWrapper for MyInstance {\n"
        s += "///   fn get_extensions(&self) -> &vk::InstanceExtensions {\n"
        s += "///     &self.extensions\n"
        s += "///   }\n"
        s += "/// }\n"
        s += "/// \n"
        s += "/// // Implement the drop trait to clean up our instance\n"
        s += "/// impl Drop for MyInstance {\n"
        s += "///   fn drop(&mut self) {\n"
        s += "///     vk::DestroyInstance(self.get_extensions().get_handle(), ptr::null());\n"
        s += "///   }\n"
        s += "/// }\n"
        s += "/// \n"
        s += "/// //...\n"
        s += "/// // Create our custom instance\n"
        s += "/// let inst = MyInstance::new();\n"
        s += "/// \n"
        s += "/// // Now we can use instance level vulkan commands directly on our instance\n"
        s += "/// // Note that we do not need to provide the instance handle to EnumeratePhysicalDevices,\n"
        s += "/// // because the InstanceWrapper takes care of that\n"
        s += "/// let mut num_devices: u32 = 0;\n"
        s += "/// inst.EnumeratePhysicalDevices(&mut num_devices, ptr::null_mut());\n"
        s += "/// println!(\"num devices:  {}\", num_devices);\n"
        s += "/// # }\n"
        s += "/// ```\n"
        s += "#[deprecated(since = \"0.1.7\", note = \"InstanceWrapper and DeviceWrapper lack handling physical devices and might be replaced or even removed in the future\")]\n"
        s += "pub trait InstanceWrapper {\n"
        s += "  fn get_extensions(&self) -> &InstanceExtensions;\n"
        s += "  fn get_handle(&self) -> Instance {\n"
        s += "    self.get_extensions().get_handle()\n"
        s += "  }\n"
        for name in self.orderedcmds:
            if not self.used[name]: continue
            s += trait_default(self.commands[name], "instance")
        s += "}\n"

        s += "\n"
        s += "\n"
        s += "\n"
        s += "/// Trait with default pass-through implementations for vulkan commands associated with a device\n"
        s += "/// # ATTENTION - This trait might be changed or removed in the future\n"
        s += "///\n"
        s += "/// # Details\n"
        s += "/// This trait only requires the implementation of [get_extensions](trait.DeviceWrapper.html#tymethod.get_extensions).\n"
        s += "/// Device level vulkan commands are then passed through to their respective core or device extension function pointer.\n"
        s += "/// If the vulkan command required a `Device` parameter, the wrapper will drop this parameter, since the\n"
        s += "/// device handle can be provided by the [DeviceExtensions](struct.DeviceExtensions.html).\n"
        s += "///\n"
        s += "/// Example\n"
        s += "/// The DeviceWrapper uses the same pattern as the [InstanceWrapper](trait.InstanceWrapper.html), look [here for an example](trait.InstanceWrapper.html#examples).\n"
        s += "#[deprecated(since = \"0.1.7\", note = \"InstanceWrapper and DeviceWrapper lacks handling physical devices and might be removed or replaced in the future\")]\n"
        s += "pub trait DeviceWrapper {\n"
        s += "  fn get_extensions(&self) -> &DeviceExtensions;\n"
        s += "  fn get_handle(&self) -> Device {\n"
        s += "    self.get_extensions().get_handle()\n"
        s += "  }\n"
        for name in self.orderedcmds:
            if not self.used[name]: continue
            s += trait_default(self.commands[name], "device")
        s += "}\n"

        return s


