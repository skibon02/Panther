#version 320 es

layout (location = 0) in vec2 position;

out vec2 v_texcoord;
out vec2 v_position;

uniform float y_ratio;

void main() {
    gl_Position = vec4(position, 0.0, 1.0);

    v_position = position * 0.5 + 0.5;
    v_texcoord = v_position;

    v_position.y *= y_ratio;
}