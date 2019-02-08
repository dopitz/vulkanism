from collections import namedtuple

import typeid as tid

Hand = namedtuple('Hand', 'parent')

def parse_handle(t):
    if tid.isalias(t): return tid.makealias(t)
    parent = ""
    if "parent" in t.attrib:
        parent = t.attrib["parent"].split(",")[0]
    return tid.maketype(t.find("name").text, "handle", Hand(parent), [])

def write_handle(types, t):
    return "#[doc(hidden)] pub type " + types.format_type(t.name) + " = u64;\n"

def write_null_handle():
    return "#[doc(hidden)] pub const NULL_HANDLE: u64 = 0;\n"

