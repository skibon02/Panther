#version 320 es
precision highp float;

uniform vec3 color;
uniform vec4 bounds;
uniform float y_ratio;

in vec2 v_position; // normalized position where x 0..1, y 0..v_y_ratio

out vec4 fragColor;

void main() {
    float x = v_position.x;
    float y = v_position.y;

    if (x < bounds.x || y < bounds.y || x - bounds.x > bounds.z || y - bounds.y > bounds.w) {
        discard;
    }

    fragColor = vec4(color, 1.0);
}
