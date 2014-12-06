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

#ifdef USE_SSBO
    #extension GL_ARB_shader_draw_parameters: require
#endif

struct DrawInfoCore {
    uint id;
    uint matrix;
    uint material;
};

struct DrawInfoStruct {
    uint id;
    uint matrix;
    uint material;
    uint _padd;
    vec4 sphere;
};

#ifdef USE_SSBO
    layout(std430, binding=4) buffer DrawInfo {
        DrawInfoStruct info[];
    };

    layout(std430, binding=5) buffer ModelMatrix {
        mat4 model_matrix[];
    };

    mat4 get_mat(int idx) {
        return model_matrix[idx];
    }

    DrawInfoCore get_info(int idx) {
        DrawInfoStruct f_info = info[idx];
        return DrawInfoCore(f_info.id,
                            f_info.matrix,
                            f_info.material);
    }

    int get_index() {
        return gl_DrawIDARB + gl_InstanceID;
    }
#else
    uniform samplerBuffer model_matrix0;
    uniform samplerBuffer model_matrix1;
    uniform samplerBuffer model_matrix2;
    uniform samplerBuffer model_matrix3;
    uniform usamplerBuffer info_buffer;

    uniform int base_index;

    mat4 get_mat(int idx) {
        return mat4(texelFetch(model_matrix0, idx),
                    texelFetch(model_matrix1, idx),
                    texelFetch(model_matrix2, idx),
                    texelFetch(model_matrix3, idx));
    }

    DrawInfoCore get_info(int idx) {
        uvec3 f_info = texelFetch(info_buffer, idx).xyz;
        return DrawInfoCore(f_info.x,
                            f_info.y,
                            f_info.z);
    }

    int get_index() {
        return base_index + gl_InstanceID;
    }
#endif

uniform mat4 mat_view;
uniform mat4 mat_proj;

in vec3 in_position;
in vec2 in_texture;
in vec3 in_normal;

out vec2 fs_texture;
out vec3 fs_normal;
flat out uint fs_object_id;
flat out uint fs_material_id;

void main() {
    int idx = get_index();
    DrawInfoCore info = get_info(idx);
    mat4 mat_model = get_mat(int(info.matrix));

    vec4 normal = mat_model * vec4(in_normal, 0.);
    gl_Position = mat_proj * mat_view * mat_model * vec4(in_position, 1.);

    fs_texture = in_texture;
    fs_normal = normalize(normal).xyz;
    fs_material_id = info.material;
    fs_object_id = info.id;
}
