# rafx demo SSAO todo

* depth/normal prepass
  * [DONE] modify depth prepass shaders to write out a view space normal texture in a fragment shader as well
  * [DONE] modify depth prepass material to add fragment shader
  * [DONE] wire up required camera matrices
  * [DONE] create the normal texture
  * [DONE] wire up the normal texture to the depth prepass
  * [DONE] how can i display the normal texture to verify it is correct?
    * [NO] ~~rewire the render graph~~
    * just run in ~~xcode~~ renderdoc!
* ssao pass
  * [DONE] create AO texture
  * [DONE] wire up depth and view space normal textures
  * [DONE] wire up required camera matrices
* opaque pass
  * wire up AO texture
  * modify shaders to apply ambient occlusion