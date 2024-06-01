#version 300 es
precision highp float;

in vec2 position;
in vec2 texcoord;

uniform float y_ratio;

out vec2 v_position;
out vec2 v_texcoord;

void main() {

    v_position = position; // 0..1

    gl_Position = vec4(position.x * 2.0 - 1.0, position.y * 2.0 / y_ratio - 1.0, 0.0, 1.0);

    v_texcoord = texcoord;
}