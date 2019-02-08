from collections import namedtuple

import typeid as tid
import decl

Str = namedtuple('Str', 'member impldbg')


def parse_struct(t):
    if tid.isalias(t): return tid.makealias(t)

    member = []
    requires = []
    for m in t.findall("./member"):
        d = decl.parse_decl(m)
        member.append(d)
        if not d.type in tid.ctypes and not d.type in requires: requires.append(d.type)

    return tid.maketype(t.attrib["name"], t.attrib["category"], Str(member, False), requires)

def parse_impldbg(types):
    def can_impldbg(t):
        for m in t.type.member:
            if "[" in m.decl: return False
            if m.type in types.categories["funcpointer"]: return False

        for r in t.requires:
            if r in types.categories["union"]: return False
            if r in types.categories["struct"] and not types.types[r].type.impldbg: return False

        return True

    found = True
    while found:
        found = False
        for name in types.categories["struct"]:
            t = types.types[name]
            if not t.type.impldbg:
                if can_impldbg(t):
                    types.types[name] = tid.maketype(name, t.category, Str(t.type.member, True), t.requires).type
                    found = True



def write_struct(types, t):
    s = "#[doc(hidden)]\n"
    s += "#[repr(C)]\n"
    if t.type.impldbg:
        s += "#[derive(Debug, Copy, Clone)]\n"
    else:
        s += "#[derive(Copy, Clone)]\n"
    s += "pub " + t.category + " " + types.format_type(t.name) + " {\n"
    
    for m in t.type.member:
        s += "  pub " + decl.write_decl(decl.resolve_decl(types, m)) + ",\n"
        
    s += "}\n"
    return s
