#version 440
#extension GL_ARB_shader_draw_parameters: require

struct DrawInfoStruct {
    int id;
    int matrix;
    int material;
    int _padd;
    vec4 sphere;
};

layout(std430, binding=4) buffer DrawInfo {
    DrawInfoStruct info[];
};

layout(std430, binding=5) buffer ModelMatrix {
    mat4 model_matrix[];
};

uniform mat4 mat_view;
uniform mat4 mat_proj;

in vec3 in_position;
in vec2 in_texture;
in vec3 in_normal;
in uint in_draw_id;

out vec2 fs_texture;
out vec3 fs_normal;
flat out uint fs_object_id;
flat out uint fs_material_id;

void main() {
    int idx = gl_DrawIDARB + gl_InstanceID;
    mat4 mat_model = model_matrix[info[idx].matrix];

    vec4 normal = mat_model * vec4(in_normal, 0.);
    gl_Position = mat_proj * mat_view * mat_model * vec4(in_position, 1.);

    fs_texture = in_texture;
    fs_normal = normalize(normal).xyz;
    fs_material_id = info[idx].material;
    fs_object_id = info[idx].id;
}