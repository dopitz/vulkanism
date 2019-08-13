vk::pipes::pipeline! {
  stage = {
    ty = "vert",
    glsl = "src/style/simple/pipeline/border.vert",
  }

  stage = {
    ty = "frag",
    glsl = "src/style/simple/pipeline/color.frag",
  }

  dset_name[0] = "DsViewport",
  dset_name[1] = "DsStyle",
}
