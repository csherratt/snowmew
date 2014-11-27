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

struct q4 {
    float s, x, y, z;
};

struct f3 {
    float x, y, z;
};

struct f4 {
    float x, y, z, w;
};

typedef struct q4 q4;
typedef struct f4 f4;
typedef struct f3 f3;

struct mat4 {
    float4 x, y, z, w;
};

struct transform {
    float scale;
    q4 rot;
    f3 pos;
};

typedef struct mat4 Matrix4;
typedef struct transform Transform3D;

#define DOT(OUT, A, B, i, j) \
    OUT.j.i = A.x.i * B.j.x + \
    A.y.i * B.j.y + \
    A.z.i * B.j.z + \
    A.w.i * B.j.w

Matrix4
mult_m(const Matrix4 a, const Matrix4 b) {
    Matrix4 out;

    DOT(out, a, b, x, x);
    DOT(out, a, b, x, y);
    DOT(out, a, b, x, z);
    DOT(out, a, b, x, w);

    DOT(out, a, b, y, x);
    DOT(out, a, b, y, y);
    DOT(out, a, b, y, z);
    DOT(out, a, b, y, w);

    DOT(out, a, b, z, x);
    DOT(out, a, b, z, y);
    DOT(out, a, b, z, z);
    DOT(out, a, b, z, w);

    DOT(out, a, b, w, x);
    DOT(out, a, b, w, y);
    DOT(out, a, b, w, z);
    DOT(out, a, b, w, w);

    return out;
}

Matrix4
transform_to_matrix4(global Transform3D *trans) {
    Matrix4 mat;

    float x2 = trans->rot.x + trans->rot.x;
    float y2 = trans->rot.y + trans->rot.y;
    float z2 = trans->rot.z + trans->rot.z;

    float xx2 = x2 * trans->rot.x;
    float xy2 = x2 * trans->rot.y;
    float xz2 = x2 * trans->rot.z;

    float yy2 = y2 * trans->rot.y;
    float yz2 = y2 * trans->rot.z;
    float zz2 = z2 * trans->rot.z;

    float sy2 = y2 * trans->rot.s;
    float sz2 = z2 * trans->rot.s;
    float sx2 = x2 * trans->rot.s;

    mat.x.x = (1. - yy2 - zz2) * trans->scale;
    mat.x.y = (xy2 + sz2) * trans->scale;
    mat.x.z = (xz2 - sy2) * trans->scale;
    mat.x.w = 0.;

    mat.y.x = (xy2 - sz2) * trans->scale;
    mat.y.y = (1. - xx2 - zz2) * trans->scale;
    mat.y.z = (yz2 + sx2) * trans->scale;
    mat.y.w = 0.;

    mat.z.x = (xz2 + sy2) * trans->scale;
    mat.z.y = (yz2 - sx2) * trans->scale;
    mat.z.z = (1. - xx2 - yy2) * trans->scale;
    mat.z.w = 0.;

    mat.w.x = trans->pos.x;
    mat.w.y = trans->pos.y;
    mat.w.z = trans->pos.z;
    mat.w.w = 1.;

    return mat;
}

Matrix4
get_mat4(global float4* x, global float4* y, global float4* z, global float4* w, uint idx) {
    Matrix4 mat;
    mat.x = x[idx];
    mat.y = y[idx];
    mat.z = z[idx];
    mat.w = w[idx];
    return mat;
}

void
set_mat4(global float4* x, global float4* y, global float4* z, global float4* w, uint idx, Matrix4 mat) {
    x[idx] = mat.x;
    y[idx] = mat.y;
    z[idx] = mat.z;
    w[idx] = mat.w;
}

kernel void
calc_vec4(global Transform3D *t,
          global int *parent,
          global float4* x,
          global float4* y,
          global float4* z,
          global float4* w,
          int limit) {

    int idx = get_global_id(0);
    if (idx >= limit) return;

    int next = parent[idx];
    Matrix4 m = transform_to_matrix4(&t[idx]);
    while (next != ~0) {
        m = mult_m(transform_to_matrix4(&t[next]), m);
        next = parent[next];
    }
    set_mat4(x, y, z, w, idx, m);
}

kernel void
calc_mat(global Transform3D *t,
          global int *parent,
         global struct mat4* mat,
         int limit) {
    int idx = get_global_id(0);
    if (idx >= limit) return;

    int next = parent[idx];
    Matrix4 m = transform_to_matrix4(&t[idx]);
    while (next != ~0) {
        m = mult_m(transform_to_matrix4(&t[next]), m);
        next = parent[next];
    }
    mat[idx] = m;
}
