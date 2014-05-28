#version 410

uniform sampler2D position;

uniform sampler2D normal;
uniform sampler2D ka;
uniform sampler2D kd;
uniform sampler2D ks;
uniform sampler2D ns_ni;

uniform vec4 point_light_center;
uniform vec4 point_light_color;
uniform float intensity;

in vec2 TexPos;
out vec4 color;

void main() {

}
