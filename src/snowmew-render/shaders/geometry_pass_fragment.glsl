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