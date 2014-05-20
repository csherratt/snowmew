#version 400

struct texture_ptr {
    int which_atlas;
    int which_index;
};

struct material {
    vec4 kd;
    texture_ptr kd_map;
};

layout(std140) uniform Materials {
    material materials[100];
};

uniform sampler2D position;
uniform sampler2D uv;
uniform sampler2D normal;
uniform usampler2D pixel_drawn_by;

uniform sampler2DArray atlases[12];

in vec2 TexPos;
out vec4 color;

void main() {
    color = vec4(TexPos.x, TexPos.y, 0., 1.);
    uint mat = texture(pixel_drawn_by, TexPos).y;
    uint object = texture(pixel_drawn_by, TexPos).x;

    if (object == 0) {
        color = vec4(1., 0.1, 0.5, 1.);
    } else {
        int kd_text = materials[mat].kd_map.which_atlas;
        int kd_idx = materials[mat].kd_map.which_index;
        if (kd_text == 0) {
            color = vec4(materials[mat].kd);
        } else {
            vec2 uv_value = texture(uv, TexPos).xy;
            color = texture(atlases[kd_text-1], vec3(uv_value, float(kd_idx)));
            //color = vec4(float(kd_text)/255., float(kd_idx)/255., float(mat)/255., 1.);
        }
    }
}