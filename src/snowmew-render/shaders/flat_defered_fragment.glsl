#version 400

struct texture_ptr {
    int which_atlas;
    int which_index;
};

struct material {
    vec3 kd;
    texture_ptr kd_map;
};

layout(std140) uniform Materials {
    material materials[2048];
};

uniform sampler2D position;
uniform sampler2D uv;
uniform sampler2D normal;
uniform usampler2D pixel_drawn_by;

uniform sampler2DArray atlases[64];

in vec2 TexPos;
out vec4 color;

void main() {
    color = vec4(TexPos.x, TexPos.y, 0., 1.);
    uvec2 material = texture(pixel_drawn_by, TexPos).xy;

    if (material.x == 0) {
        color = vec4(1., 0.1, 0.5, 1.);
    } else {
        int kd_text = materials[material.y].kd_map.which_atlas-1;
        int kd_idx = materials[material.y].kd_map.which_index;
        if (kd_text == -1) {
            color = vec4(materials[material.y].kd, 1.);
        } else {
            vec2 uv_value = texture(uv, TexPos).xy;
            color = texture(atlases[kd_text],
                            vec3(uv_value, float(kd_idx)));
        }
    }
}