# rafx demo SSAO todo

* depth/normal prepass
  * [DONE] modify depth prepass shaders to write out a view space normal texture in a fragment shader as well
  * [DONE] modify depth prepass material to add fragment shader
  * [DONE] wire up required camera matrices
  * create the normal texture
  * wire up the normal texture to the depth prepass
  * how can i display the normal texture to verify it is correct?
    * [NO] ~~rewire the render graph~~
    * just run in xcode!
* ssao pass
  * create AO texture
  * wire up depth and view space normal textures
  * wire up required camera matrices