#version 320 es
precision highp float;

uniform sampler2D tex;
uniform vec4 bounds;

in vec2 v_position;

out vec4 fragColor;

void main() {
    float x = (v_position.x - bounds.x) / bounds.z;
    float y = 1.0 - (v_position.y - bounds.y) / bounds.w;

    if (x < 0.0 || y < 0.0 || x > 1.0 || y > 1.0) {
        discard;
    }

    fragColor = texture(tex, vec2(x, y));
}