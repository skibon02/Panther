#version 320 es
precision highp float;

in vec2 position;

uniform float y_ratio;

out vec2 v_position;

void main() {
    gl_Position = vec4(position, 0.0, 1.0);

    v_position = position * 0.5 + 0.5; // 0..1
    v_position.y *= y_ratio;
}