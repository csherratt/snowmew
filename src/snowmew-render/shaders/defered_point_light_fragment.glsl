//   Copyright 2014 Colin Sherratt
//
//   Licensed under the Apache License, Version 2.0 (the "License");
//   you may not use this file except in compliance with the License.
//   You may obtain a copy of the License at
//
//       http://www.apache.org/licenses/LICENSE-2.0
//
//   Unless required by applicable law or agreed to in writing, software
//   distributed under the License is distributed on an "AS IS" BASIS,
//   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//   See the License for the specific language governing permissions and
//   limitations under the License.

#define ATLAS_SIZE 8

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
uniform sampler2D dxdt;

uniform sampler2DArray atlas[ATLAS_SIZE];
uniform int atlas_base;

uniform vec4 viewport;
uniform mat4 mat_proj;
uniform mat4 mat_view;
uniform mat4 mat_inv_proj;
uniform mat4 mat_inv_view;

in vec2 TexPos;
out vec4 color;

fetch_result fetch_material(vec4 d, ivec2 map, vec2 uv_value, vec2 xy, vec2 zw) {
    if (map.x >= atlas_base && map.x < atlas_base + ATLAS_SIZE) {
        vec4 text = textureGrad(atlas[map.x-atlas_base],
                               vec3(uv_value, float(map.y)),
                               xy, zw);
        return fetch_result(vec4(text.xyz, 1.), true);
    } else if (map.x == -1 && atlas_base == 0) {
        return fetch_result(d, true);
    } else {
        return fetch_result(vec4(0., 0., 0., 0.), false);
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

    return mat_inv_view * mat_inv_proj * clip_pose;
}

void main() {
    uvec2 object = texture(pixel_drawn_by, TexPos).xy;
    vec2 uv_value = texture(uv, TexPos).xy;
    vec4 dxdy = texture(dxdt, TexPos); 
    fetch_result ka = fetch_material(materials[object.y].ka,
                                     materials[object.y].ka_map,
                                     uv_value, dxdy.xy, dxdy.zw);
    fetch_result kd = fetch_material(materials[object.y].kd,
                                     materials[object.y].kd_map,
                                     uv_value, dxdy.xy, dxdy.zw);
    fetch_result ks = fetch_material(materials[object.y].ks,
                                     materials[object.y].ks_map,
                                     uv_value, dxdy.xy, dxdy.zw);
    vec4 pos = calc_pos_from_window(vec3(gl_FragCoord.x,
                                         gl_FragCoord.y,
                                         texture(depth, TexPos).x));
    vec4 surface_normal = vec4(texture(normal, TexPos).xyz, 0.);
    vec4 eye_pos = mat_inv_view * vec4(0., 0., 0., 1.);
    vec4 eye_to_point_normal = normalize(eye_pos - pos);

    vec4 c = vec4(0);
    if (ka.found) {
        c = ka.value * 0.2;
    }

    for (int i = 0; i < point_count; i++) {
        vec4 delta = point_lights[i].position - pos;
        float dist = length(delta);
        dist = 1. / (dist*dist);
        vec4 light_to_point_normal = normalize(delta);
        if (kd.found) {
            c += kd.value * point_lights[i].color * dist * 
                        max(0, dot(light_to_point_normal, surface_normal));
        }

        if (ks.found) {
            vec4 h = normalize(light_to_point_normal + eye_to_point_normal);
            float ns = materials[object.y].ns;
            float facing = 0;
            if (dot(light_to_point_normal, surface_normal) > 0) {
                facing = 1;
            }
            float factor = pow(max(0, dot(h, surface_normal)), ns);
            if (factor > 0) {
                c += ks.value * direction_lights[i].color * facing * dist * factor;
            }
        }
    }

    for (int i = 0; i < direction_count; i++) {
        vec4 light_to_point_normal = direction_lights[i].normal;
        if (kd.found) {
            c += kd.value * direction_lights[i].color * 
                     max(0, dot(light_to_point_normal, surface_normal));
        }

        if (ks.found) {
            vec4 h = normalize(light_to_point_normal + eye_to_point_normal);
            float ns = materials[object.y].ns;
            float facing = 0;
            if (dot(light_to_point_normal, surface_normal) > 0) {
                facing = 1;
            }
            float factor = pow(max(0, dot(h, surface_normal)), ns);
            if (factor > 0) {
                c += ks.value * direction_lights[i].color * facing * factor;
            }
        }
    }

    color = vec4(c.xyz, 1);
}