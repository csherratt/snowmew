#version 430

layout(location = 0) uniform mat4 mat_proj_view;
layout(location = 1) uniform int instance[512];

layout(std430, binding = 3) buffer MyBuffer
{
    mat4 model_matrix[];
};

in vec3 in_position;
in vec2 in_texture;
in vec3 in_normal;

out vec3 fs_position;
out vec2 fs_texture;
out vec3 fs_normal;

void main() {
    int id = instance[gl_InstanceID];
    gl_Position = mat_proj_view * model_matrix[id] * vec4(in_position, 1.);
    fs_position = model_matrix[id] * vec4(in_position, 1.);
    fs_texture = in_texture;
    fs_normal = in_normal;
    fs_material_id = material_id;
    fs_object_id = object_id;
}