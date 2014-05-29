#version 410

#define ATLAS_SIZE 12

struct material {
    vec4 ka;
    vec4 kd;
    vec4 ks;

    ivec2 ka_map;
    ivec2 kd_map;
    ivec2 ks_map;

    float ns;
    float ni;
};

struct fetch_result {
    vec4 value;
    bool found;
};

struct point {
    vec4 color;
    vec4 position;
};

struct direction {
    vec4 color;
    vec4 normal;
};

layout(std140) uniform Lights {
    int point_count;
    int direction_count;
    int _padding0; 
    int _padding1;
    point point_lights[480];
    direction direction_lights[8];
};

layout(std140) uniform Materials {
    material materials[512];
};

uniform sampler2D normal;
uniform sampler2D uv;
uniform usampler2D pixel_drawn_by;
uniform sampler2D depth;

uniform sampler2DArray atlas[ATLAS_SIZE];
uniform int atlas_base;

uniform vec4 viewport;
uniform mat4 mat_proj;
uniform mat4 mat_view;

in vec2 TexPos;
out vec4 color;

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

vec4 calc_pos_from_window(vec3 window_space) {
    vec2 depthrange = vec2(0., 1.);
    vec3 ndc_pos;
    ndc_pos.xy = ((2.0 * window_space.xy) - (2.0 * viewport.xy)) / (viewport.zw) - 1;
    ndc_pos.z = (2.0 * window_space.z - depthrange.x - depthrange.y) /
               (depthrange.y - depthrange.x);

    vec4 clip_pose;
    clip_pose.w = mat_proj[3][2] / (ndc_pos.z - (mat_proj[2][2] / mat_proj[2][3]));
    clip_pose.xyz = ndc_pos * clip_pose.w;

    return inverse(mat_view) * inverse(mat_proj) * clip_pose;
}

void main() {
    uvec2 object = texture(pixel_drawn_by, TexPos).xy;
    fetch_result ka = fetch_material(materials[object.y].ka,
                                     materials[object.y].ka_map);
    fetch_result kd = fetch_material(materials[object.y].kd,
                                     materials[object.y].kd_map);
    fetch_result ks = fetch_material(materials[object.y].ks,
                                     materials[object.y].ks_map);
    vec4 pos = calc_pos_from_window(vec3(gl_FragCoord.x,
                                         gl_FragCoord.y,
                                         texture(depth, TexPos).x));
    vec4 surface_normal = vec4(texture(normal, TexPos).xyz, 0.);
    vec4 eye_pos = inverse(mat_view) * vec4(0., 0., 0., 1.);
    vec4 eye_to_point_normal = normalize(eye_pos - pos);

    if (ka.found) {
        color = ka.value * 0.2;
    }

    for (int i = 0; i < point_count; i++) {
        vec4 delta = point_lights[i].position - pos;
        float dist = length(delta);
        dist = 1. / (dist*dist);
        vec4 light_to_point_normal = normalize(delta);
        if (kd.found) {
            color += kd.value * point_lights[i].color * dist * 
                        max(0, dot(light_to_point_normal, surface_normal));
        }

        if (ks.found) {
            vec4 h = normalize(light_to_point_normal + eye_to_point_normal);
            float ns = materials[object.y].ns;
            float facing = 0;
            if (dot(light_to_point_normal, surface_normal) > 0) {
                facing = 1;
            }
            color += ks.value * point_lights[i].color * facing *
                    dist * pow(max(0, dot(h, surface_normal)), ns);
        }
    }

    for (int i = 0; i < direction_count; i++) {
        vec4 light_to_point_normal = direction_lights[i].normal;
        if (kd.found) {
            color += kd.value * direction_lights[i].color * 
                     max(0, dot(light_to_point_normal, surface_normal));
        }

        if (ks.found) {
            vec4 h = normalize(light_to_point_normal + eye_to_point_normal);
            float ns = materials[object.y].ns;
            float facing = 0;
            if (dot(light_to_point_normal, surface_normal) > 0) {
                facing = 1;
            }
            color += ks.value * direction_lights[i].color * facing *
                     pow(max(0, dot(h, surface_normal)), ns);
        }
    }
}