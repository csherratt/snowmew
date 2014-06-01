#version 400

in vec2 fs_texture;
in vec3 fs_normal;
flat in uint fs_object_id;
flat in uint fs_material_id;

out vec2 out_uv;
out vec3 out_normal;
out uvec4 out_material;
out vec4 out_dxdt;

void main() {
    out_uv = fs_texture;
    out_normal = fs_normal;
    out_material = uvec4(fs_object_id, fs_material_id, 0, 0);
    out_dxdt = vec4(dFdx(fs_texture), dFdy(fs_texture));
}