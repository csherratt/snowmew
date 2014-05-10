#version 400

in vec3 fs_position;
in vec2 fs_texture;
in vec3 fs_normal;
flat in uint fs_object_id;
flat in uint fs_material_id;

out vec4 out_position;
out vec2 out_uv;
out vec3 out_normal;
out vec4 out_material;

void main() {
    uint mask = 0xFFFF;
    out_position = vec4(fs_position, gl_FragCoord.z);
    out_uv = fs_texture;
    out_normal = fs_normal;
    out_material = vec4(float(fs_material_id) / 65535., float(fs_object_id) / 65535., 1., 1.);
}