#version 400

uniform vec3 mat_color[128];

uniform sampler2D position;
uniform sampler2D uv;
uniform sampler2D normal;
uniform sampler2D pixel_drawn_by;

in vec2 TexPos;
out vec4 color;

void main() {
    ivec2 material = ivec2(texture(pixel_drawn_by, TexPos).xy * 65536.);
    bool edge = 
            (material != ivec2(textureOffset(pixel_drawn_by, TexPos, ivec2( 0,  1)).xy * 65536.)) ||
            (material != ivec2(textureOffset(pixel_drawn_by, TexPos, ivec2( 0, -1)).xy * 65536.)) ||
            (material != ivec2(textureOffset(pixel_drawn_by, TexPos, ivec2( 1,  0)).xy * 65536.)) ||
            (material != ivec2(textureOffset(pixel_drawn_by, TexPos, ivec2(-1,  0)).xy * 65536.)) ||
            (material != ivec2(textureOffset(pixel_drawn_by, TexPos, ivec2( 1,  1)).xy * 65536.)) ||
            (material != ivec2(textureOffset(pixel_drawn_by, TexPos, ivec2(-1, -1)).xy * 65536.)) ||
            (material != ivec2(textureOffset(pixel_drawn_by, TexPos, ivec2( 1, -1)).xy * 65536.)) ||
            (material != ivec2(textureOffset(pixel_drawn_by, TexPos, ivec2(-1,  1)).xy * 65536.));

    if (material.x == 0) {
        color = vec4(0., 0., 0., 1.);
    } else {
        color = vec4(mat_color[material.x], 1.);
    }

    if (edge) {
        color *= 0.5;
    }
}