#version 300 es

precision mediump float;

// vec3 v_color = vec3(1.0, 0.3, 0.1);

in vec2 v_texcoord;
in vec2 v_position; // normalized position where x 0..1, y 0..v_y_ratio

uniform sampler2D u_texture;
uniform vec3 u_circle;

out vec4 fragColor;

void main() {
    float dist = distance(v_position, u_circle.xy);
    if (dist > u_circle.z) {
        fragColor = vec4(0.0, 0.0, 0.0, 0.0);
    } else {
        fragColor = texture(u_texture, v_texcoord);
    }
}