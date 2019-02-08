import typeid as tid
import struct
import decl

def get_types():
    return [tid.maketype("Display", "exttype", "c_void", []),
        tid.maketype("Window", "exttype", "c_ulong", []),
        tid.maketype("RROutput", "exttype", "c_ulong", []),
        tid.maketype("VisualID", "exttype", "u32", []),

        tid.maketype("xcb_connection_t", "exttype", "c_void", []),
        tid.maketype("xcb_window_t", "exttype", "u32", []),
        tid.maketype("xcb_visualid_t", "exttype", "u32", []),

        tid.maketype("wl_display", "exttype", "c_void", []),
        tid.maketype("wl_surface", "exttype", "c_void", []),

        tid.maketype("HINSTANCE", "exttype", "*mut c_void", []),
        tid.maketype("HWND", "exttype", "*mut c_void", []),
        tid.maketype("HANDLE", "exttype", "*mut c_void", []), 
        tid.maketype("DWORD", "exttype", "c_ulong", []), 
        tid.maketype("LPVOID", "exttype", "*mut c_void", []), 
        tid.maketype("BOOL", "exttype", "i32", []), 
        tid.maketype("LPCWSTR", "exttype", "*mut c_ushort", []), 
        tid.maketype("win32_SECURITY_ATTRIBUTES", "struct",
            struct.Str([
                decl.Decl("nLenght", "DWORD", "DWORD"),
                decl.Decl("lpSecurityDescriptor", "LPVOID", "LPVOID"),
                decl.Decl("bInheritHandle", "BOOL", "BOOL")], False), ["DWORD", "LPVOID", "BOOL"]), 
        tid.ParseType("SECURITY_ATTRIBUTES", "win32_SECURITY_ATTRIBUTES"),

        tid.maketype("ANativeWindow", "exttype", "c_void", []),
        tid.maketype("AHardwareBuffer", "exttype", "c_void", []), # TODO: type?

        tid.maketype("zx_handle_t", "exttype", "u32", []),
        ]

def write_type(types, t):
    return "#[doc(hidden)] pub type " + types.format_type(t.name) + " = " + t.type + ";\n"
