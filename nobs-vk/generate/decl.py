from collections import namedtuple

import enum

Decl = namedtuple('Decl', 'name type decl')


def parse_typemods(decl, name, typename):
    # remove all but type modifier
    if name.find(typename) != -1:
        decl = decl.replace(name, "")
        decl = decl.replace(typename, "")
    else:
        decl = decl.replace(typename, "")
        decl = decl.replace(name, "")

    decl = decl.replace("struct", "")
    decl = decl.replace(" ", "")
    
    # this is an array type (int a[3])
    if "[" in decl:
        decl = decl.replace("const", "")
        n = decl.strip("[]")
        if not n.isdigit():
            n = enum.format_enum_name(n) + " as usize"
        return "[#typename; " + n + "]"
  
    # pointer or value type
    else:
        rt = ""
        p = decl.find("*", 0)
        c = decl.find("const", 0)
        while p != -1:
            if p != -1:
                if c != -1:
                    rt += "*const "
                else:
                    rt += "*mut "
            p = decl.find("*", p + 1)
            c = decl.find("const", c + 1)
    
        return rt + "#typename"


def parse_decl(d):
    name = d.find("name").text 
    ctype = d.find("type").text
    decl = "".join(d.itertext())
    comment = d.find("comment")
    if comment != None:
        decl = decl.replace(comment.text, "")

    return Decl(name, ctype, parse_typemods(decl, name, ctype))

def resolve_decl(types, d):
    return Decl("typ" if d.name == "type" else d.name, types.format_type(d.type), d.decl)

def write_decl(d):
    s = d.decl;
    return d.name + ": " + s.replace("#typename", d.type)

