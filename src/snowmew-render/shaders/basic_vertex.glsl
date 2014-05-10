#version 150

uniform mat4 mat_model;
uniform mat4 mat_proj_view;
uniform uint object_id;
uniform uint material_id;

in vec3 in_position;
in vec2 in_texture;
in vec3 in_normal;

out vec3 fs_position;
out vec2 fs_texture;
out vec3 fs_normal;
flat out uint fs_object_id;
flat out uint fs_material_id;

void main() {
    gl_Position = mat_proj_view * mat_model * vec4(in_position, 1.);
    vec4 pos = mat_model * vec4(in_position, 1.);
    fs_position = pos.xyz / pos.w;
    fs_texture = in_texture;
    fs_normal = in_normal;
    fs_object_id = object_id;
    fs_material_id = material_id;
}