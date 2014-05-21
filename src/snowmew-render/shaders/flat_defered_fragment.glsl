#version 410

#define ATLAS_SIZE 12

layout(std140) uniform Materials {
    struct material {
        vec4 kd;
        ivec2 kd_map;
    } materials[100];
};

uniform sampler2D position;
uniform sampler2D uv;
uniform sampler2D normal;
uniform usampler2D pixel_drawn_by;

uniform sampler2DArray atlas[ATLAS_SIZE];
uniform int atlas_base;

in vec2 TexPos;
out vec4 color;

void main() {
    uvec2 object = texture(pixel_drawn_by, TexPos).xy;
    ivec2 kd_map = materials[object.y].kd_map;

    if (kd_map.x == -1) {
        color = materials[object.y].kd;
    } else if (kd_map.x >= atlas_base && kd_map.x < atlas_base + ATLAS_SIZE) {
        vec2 uv_value = texture(uv, TexPos).xy;
        color = vec4(texture(atlas[kd_map.x], vec3(uv_value, float(kd_map.y))).xyz, 1.);
    } else {
        discard;
    }
}