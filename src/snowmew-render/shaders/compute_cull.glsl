#version 440

struct DrawInfoStruct {
    int id;
    int matrix;
    int material;
    int _padd;
    vec4 sphere;
};

struct DrawElementCommand {
    int count;
    int instance_count;
    int first_index;
    int base_vertex;
    int base_instance;
};

layout(std430, binding=0) buffer DrawInfo {
    DrawInfoStruct info[];
};

layout(std430, binding=1) buffer ModelMatrix {
    mat4 model_matrix[];
};

layout(std430, binding=2) buffer DrawCommand {
    DrawElementCommand commands[];
};

layout(local_size_x = 64, local_size_y = 1) in;

uniform vec4 plane[6];
uniform int max_id;

void main() {
    uint id = gl_WorkGroupID.x + gl_WorkGroupID.y * 256;
    bool accept = true;

    if (id < max_id) {
        // cannot call an instanced command
        if (commands[id].instance_count > 1) {
            return;
        }

        mat4 mat = model_matrix[info[id].matrix];
        vec4 sphere_center = mat * vec4(info[id].sphere.xyz, 1.);
        sphere_center = sphere_center.xyzw / sphere_center.w; 
        float sphere_radius = length(vec4(1/sqrt(3), 1/sqrt(3), 1/sqrt(3), 0.) * mat) * info[id].sphere.w;

        for (int i=0; i<6; i++) {
            if (dot(plane[i], sphere_center) + sphere_radius < 0.) {
                accept = false;
            }
        }

        commands[id].instance_count = accept ? 1 : 0;
    }
}