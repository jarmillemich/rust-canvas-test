#version 300 es

// Projection from screen space
uniform vec2 u_resolution;
uniform vec2 u_center;
uniform float u_radius;

in vec2 position;
uniform vec4 u_color;

out vec2 v_coord;
out vec4 v_color;
out vec2 v_center;
out float v_radius;

void main() {

    gl_Position = vec4((position / u_resolution) * 2.0 - 1.0, 0., 1.);
    v_color = u_color;
    v_center = u_center;
    v_coord = position;
    v_radius = u_radius;
}