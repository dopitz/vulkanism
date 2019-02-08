import enum
import typeid as tid

ignored_types = ["vk_platform",
        "VK_API_VERSION",
        "VK_API_VERSION_1_0",
        "VK_API_VERSION_1_1",
        "VK_VERSION_MAJOR",
        "VK_VERSION_MINOR",
        "VK_VERSION_PATCH",
        "VK_HEADER_VERSION",
        "VK_NULL_HANDLE"]

def parse_require(r, types, cmds, extnumber, extnames, feature):
    if r.tag == "type":
        if r.attrib["name"] in ignored_types:
            return
    
        types.require(r.attrib["name"])
    
    elif r.tag == "enum":
        if r.attrib["name"].endswith("EXTENSION_NAME"):
            extnames.type.member.append(enum.EnMember(r.attrib["name"], r.attrib["value"], "&str"))
        else:
            enum.parse_extend_enum(r, types, extnumber)

    
    elif r.tag == "command":
        reqtypes = cmds.require(r.attrib["name"], feature)
        for t in reqtypes:
            types.require(t)


def is_empty_extension(e):
    if "supported" in e.attrib:
        if e.attrib["supported"] == "disabled": return True

    for req in e.findall("./require"):
        for r in req:
            if r.tag == "type": return False
            if r.tag == "command": return False
            if r.tag == "enum":
                if not (r.attrib["name"].endswith("SPEC_VERSION") or r.attrib["name"].endswith("EXTENSION_NAME")): return False

    return True



def require_feature(root, version, types, cmds):
    for feature in root.findall("./feature"):
        if feature.attrib["name"] != version:
            continue

        for req in feature:
            for r in req:
                parse_require(r, types, cmds, "", None, tid.Feature("core", version))



def require_extension(root, name, types, cmds, extnames):
    for ext in root.findall("./extensions/extension"):
        if ext.attrib["name"] != name:
            continue

        extnumber = ""
        if "number" in ext.attrib:
            extnumber = ext.attrib["number"]

        for req in ext.findall("./require"):
            for r in req:
                version = "VK_VERSION_1_0"
                if "feature" in r.attrib:
                    version = r.attrib["feature"]
                parse_require(r, types, cmds, extnumber, extnames, tid.Feature(ext.attrib["type"], version))
