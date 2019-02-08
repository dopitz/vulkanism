import xml.etree.ElementTree as ET

from collections import namedtuple

import decl
import basetype
import handle
import enum
import struct
import funcptr
import exttypes


Type = namedtuple('Type', 'name category type requires')
ParseType = namedtuple('Parsetype', 'alias type')

def maketype(name, category, type, requires):
    return ParseType("", Type(name, category, type, requires))

def isalias(t):
    return "alias" in t.attrib

def makealias(t):
    return ParseType(t.attrib["name"], t.attrib["alias"])

Feature = namedtuple('Feature', 'entry version')


ctypes = {
    "void":        "c_void",
    "char":        "c_char",
    "int8_t":      "i8",
    "int32_t":     "i32",
    "int":         "i32",
    "int64_t":     "i64",
    "uint8_t":     "u8",
    "uint16_t":    "u16",
    "uint32_t":    "u32",
    "uint64_t":    "u64",
    "size_t":      "usize",
    "float":       "f32",
    "double":      "f64"
    }


class Types:
    ordered_types = []

    types = {}
    categories = {}
    used = {}

    aliases = {}

    def __init__(self, root):

        def add_type(t):
            if t.alias:
                self.aliases[t.alias] = t.type
                return

            t = t.type

            if not t.name in self.types: self.ordered_types.append(t.name)
            if not t.category in self.categories: self.categories[t.category] = []
            self.categories[t.category].append(t.name)
            self.types[t.name] = t
            self.used[t.name] = False

        add_category = {}
        add_category["basetype"] = lambda t : basetype.parse_basetype(t)
        add_category["handle"] = lambda t : handle.parse_handle(t)
        add_category["bitmask"] = lambda t : enum.parse_bitmask(t)
        add_category["enum"] = lambda t : enum.parse_enum(t)
        add_category["union"] = lambda t : struct.parse_struct(t)
        add_category["struct"] = lambda t : struct.parse_struct(t)

        for ts in root.findall("./types"):
            for t in ts.findall("./type"):
                # no category means we don't care
                if not "category" in t.attrib:
                    continue
                cat = t.attrib["category"]
                if not cat in add_category:
                    continue
                add_type(add_category[cat](t))

        for f in funcptr.get_ptrs():
            add_type(f)

        for t in exttypes.get_types():
            add_type(t)






    def get_types(self, categories, usedonly):
        def f(t):
            t = self.types[t]
            if usedonly and not self.used[t.name]: return False
            return t.category in categories

        return map(lambda t: self.types[t], filter(f, self.ordered_types))

    def resolve_type(self, t):
        if t in ctypes:
            return ctypes[t]

        if t in self.aliases:
            t = self.aliases[t]
        return t

    def format_type(self, t):
        t = self.resolve_type(t)
        if t in self.types and t not in self.categories["funcpointer"] and t not in self.categories["exttype"]:
            return t[2:]
        return t

    def require(self, name):
        if name in ctypes: return

        name = self.resolve_type(name)

        if not name in self.types:
            if not name.startswith("extern \"system\""):
                print "TYPE NOT FOUND " + name
            return

        if not self.used[name]:
            self.used[name] = True

            # recursively add required types
            for r in self.types[name].requires:
                self.require(r)


