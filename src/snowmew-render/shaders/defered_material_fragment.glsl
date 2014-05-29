#version 410

#define ATLAS_SIZE 12

struct material {
    vec4 ka;
    vec4 kd;
    vec4 ks;

    ivec2 ka_map;
    ivec2 kd_map;
    ivec2 ks_map;
    ivec2 bump_map;

    float ns;
    float ni;
};

struct fetch_result {
    vec4 value;
    bool found;
};

layout(std140) uniform Materials {
    material materials[512];
};

uniform sampler2D normal;
uniform sampler2D uv;
uniform usampler2D pixel_drawn_by;

uniform sampler2DArray atlas[ATLAS_SIZE];

uniform int atlas_base;

in vec2 TexPos;

out vec4 out_ka;
out vec4 out_kd;
out vec4 out_ks;
out vec2 out_ns_ni;
out vec4 out_bump;

fetch_result fetch_material(vec4 d, ivec2 map) {
    if (map.x == -1) {
        return fetch_result(d, true);
    } else if (map.x >= atlas_base && map.x < atlas_base + ATLAS_SIZE) {
        vec2 uv_value = texture(uv, TexPos).xy;
        vec4 text = texture(atlas[map.x], vec3(uv_value, float(map.y)));
        return fetch_result(vec4(text.xyz, 1.), true);
    } else {
        fetch_result(vec4(0., 0., 0., 0.), false);
    }
}

void main() {
    uvec2 object = texture(pixel_drawn_by, TexPos).xy;
    vec4 first_bump = texture(normal, TexPos);
    fetch_result ka = fetch_material(materials[object.y].ka,
                                     materials[object.y].ka_map);
    fetch_result kd = fetch_material(materials[object.y].kd,
                                     materials[object.y].kd_map);
    fetch_result ks = fetch_material(materials[object.y].ks,
                                     materials[object.y].ks_map);
    fetch_result bump = fetch_material(first_bump,
                                       materials[object.y].bump_map);

    if (ka.found) {
        out_ka = ka.value;
    }
    if (kd.found) {
        out_kd = kd.value;
    }
    if (ks.found) {
        out_ks = ks.value;
    }
    if (bump.found) {
        out_bump = bump.value;
    }

    out_ns_ni = vec2(materials[object.y].ns,
                     materials[object.y].ni);
}