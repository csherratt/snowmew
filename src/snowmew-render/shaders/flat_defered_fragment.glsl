#version 400

uniform vec3 mat_color[128];

uniform sampler2D position;
uniform sampler2D uv;
uniform sampler2D normal;
uniform usampler2D pixel_drawn_by;

in vec2 TexPos;
out vec4 color;

void main() {
    uvec2 material = texture(pixel_drawn_by, TexPos).xy;
    bool edge = 
            (material.x != textureOffset(pixel_drawn_by, TexPos, ivec2( 0,  1)).x) ||
            (material.x != textureOffset(pixel_drawn_by, TexPos, ivec2( 0, -1)).x) ||
            (material.x != textureOffset(pixel_drawn_by, TexPos, ivec2( 1,  0)).x) ||
            (material.x != textureOffset(pixel_drawn_by, TexPos, ivec2(-1,  0)).x) ||
            (material.x != textureOffset(pixel_drawn_by, TexPos, ivec2( 1,  1)).x) ||
            (material.x != textureOffset(pixel_drawn_by, TexPos, ivec2(-1, -1)).x) ||
            (material.x != textureOffset(pixel_drawn_by, TexPos, ivec2( 1, -1)).x) ||
            (material.x != textureOffset(pixel_drawn_by, TexPos, ivec2(-1,  1)).x);

    if (material.x == 0) {
        color = vec4(0., 0., 0., 1.);
    } else {
        color = vec4(mat_color[material.y], 1.);
    }

    if (edge) {
        color *= 0.5;
    }
}