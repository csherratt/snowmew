#version 400

uniform vec4 material_kd[400];
uniform vec2 material_kd_size[400];

uniform sampler2D position;
uniform sampler2D uv;
uniform sampler2D normal;
uniform usampler2D pixel_drawn_by;

uniform sampler2DArray atlas;

in vec2 TexPos;
out vec4 color;

void main() {
    uvec2 material = texture(pixel_drawn_by, TexPos).xy;

    if (material.x == 0) {
        color = vec4(0., 0., 0., 1.);
    } else {
        int kd_text = int(material_kd[material.y].w);
        if (kd_text == -1) {
            color = vec4(material_kd[material.y].xyz, 1.);
        } else {
            vec2 uv_value = texture(uv, TexPos).xy;
            uv_value = vec2(uv_value.x * material_kd_size[material.y].x,
                            uv_value.y * material_kd_size[material.y].y);
            vec3 pos = vec3(uv_value, float(kd_text));
            color = texture(atlas, pos);
        }
    }
}