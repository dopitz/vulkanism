import typeid as tid

def parse_basetype(t):
    return tid.maketype(t.find("name").text, "basetype", t.find("type").text, [])

def write_basetype(types, t):
    return "#[doc(hidden)] pub type " + types.format_type(t.name) + " = " + types.format_type(t.type) + ";\n"
