#version 400

uniform int instance_offset;
uniform mat4 mat_view;
uniform mat4 mat_proj;

uniform samplerBuffer mat_model0;
uniform samplerBuffer mat_model1;
uniform samplerBuffer mat_model2;
uniform samplerBuffer mat_model3;

uniform usamplerBuffer info;

in vec3 in_position;
in vec2 in_texture;
in vec3 in_normal;

out vec3 fs_position;
out vec2 fs_texture;
out vec3 fs_normal;
flat out uint fs_object_id;
flat out uint fs_material_id;

void main() {
    int instance = gl_InstanceID + instance_offset;
    uvec4 info = texelFetch(info, instance);
    int matrix_id = int(info.y);
    mat4 mat_model = mat4(texelFetch(mat_model0, matrix_id),
                          texelFetch(mat_model1, matrix_id),
                          texelFetch(mat_model2, matrix_id),
                          texelFetch(mat_model3, matrix_id));

    mat4 mat_proj_view = mat_proj * mat_view;
    mat4 mat_model_view = mat_view * mat_model;
    vec4 pos = mat_model * vec4(in_position, 1.);
    vec4 normal = mat_model_view * vec4(in_normal, 0.);

    gl_Position = mat_proj_view * pos;
    fs_position = pos.xyz / pos.w;
    fs_texture = in_texture;
    fs_normal = normal.xyz;
    fs_material_id = info.z;
    fs_object_id = info.x;
}