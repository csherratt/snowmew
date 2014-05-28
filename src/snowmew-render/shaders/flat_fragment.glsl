#version 400

in vec2 fs_texture;
in vec2 fs_normal;
flat in uint fs_object_id;
flat in uint fs_material_id;

out vec2 out_uv;
out vec2 out_normal;
out uvec4 out_material;

void main() {
    out_uv = fs_texture;
    out_normal = fs_normal;
    out_material = uvec4(fs_object_id, fs_material_id, 0, 0);
}