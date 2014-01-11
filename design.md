

# Render #

The render is a tree stage pipeline.

 * Draw list creation
 * Draw list optimization
 * Draw list execute

## Draw List creation ##
This is the first stage. This takes the drawable framebuffer and works backwards figuring out how to construct the framebuffer. Generally speaking this is taking a scene, a camera, and a viewport(that covers the entire frame). The scene is pruned so that only objects that reside inside the cone of the camera port are returned.

The list of objects are then used to create a draw list, each object has a setup function that is called. This setup function can be used to make sure any subcommands are created before the object is drawn. These subcommands could be updating textures, or indirect drawing (to textures).

## Draw List Optimization ##
The draw list is optimized, draw calls can can be reordered to minimizing geometry, texture, and shader binding. In addition to draw from the foreground to the background.

## Draw list Execute ##
 