#version 300 es

precision highp float;

uniform vec2 u_resolution;
uniform vec4 u_color;

out vec4 outColor;
in vec2 v_coord;
in vec2 v_center;
in float v_radius;

float smoothedge(float v) {
    // The second value here is probably incorrect
    return smoothstep(0.0, 4. / u_resolution.x, v);
}

float ring(vec2 p, float radius, float width) {
    return abs(length(p) - radius * 0.5) - width;
}

void main() {
    // In screen space here
    vec2 st = v_coord - v_center;
    float d = ring(st, 2. * v_radius, 0.2 * v_radius);


    //outColor = vec4(1, 0.5, 0.0, 0.5);
    float color = 1. - smoothedge(d);

    //Use actual transparency when under the alpha threshold
    if (color < 0.001) discard;

    outColor = vec4(color * u_color.xyz / 256., 1.);
    //outcolor = vec4(1., 0.5, 0.0, color);
}