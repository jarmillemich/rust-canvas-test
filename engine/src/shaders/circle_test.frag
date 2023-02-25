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

float arctan(float v) {
    // From https://stackoverflow.com/a/42542593
    // Accurate over [-1, 1] to within a couple dozen milli-radians it seems
    // Use 1/arctan(1/v) for the rest of the range
    float a = 0.0776509570923569;
    float b = -0.287434475393028;
    float c = 0.9951816816981194;

    float vv = v * v;

    return ((a * vv + b) * vv + c) * v;
}

float ring(vec2 p, float radius, float width) {
    // Makes a V wrt length, centered at half a radius, shifted down width
    // Later taken as 1 - f, so that the area under zero is now our ring
    return abs(length(p) - radius * 0.5) - width;
}

// Just testing an alternative that doesn't involve sqrt
float ring2(vec2 p, float radius, float width) {
    //return dot(p, p) > (.25 * radius * radius) ? 1. : 0.;
    // We can't really do the x-axis shift from the above without knowing the length
    // Avoid the sqrt by doing more wild math!
    float distSq = dot(p, p);
    float distQt = distSq * distSq;
    //return dot(w, w) - radius * radius * 0.25;
    // TODO We possibly have the right idea here, but we need to figure out the right scales
    //return 0.01 * distQt - 2. * distSq + radius * radius * 0.25;
    // An even better and more obvious idea, we just need to fix our intercepts/coefficients!
    return abs(distSq - radius * radius * 0.25) - width * width * 8.;
}

void main() {
    // In screen space here
    vec2 st = v_coord - v_center;
    float d = ring(st, 2. * v_radius, 0.2 * v_radius);


    //outColor = vec4(1, 0.5, 0.0, 0.5);
    float color = 1. - smoothedge(d);

    outColor = color * u_color / 256.;
    //outcolor = vec4(1., 0.5, 0.0, color);
}