#version 300 es
precision highp float;

uniform float y_ratio;
uniform sampler2D tex;

in vec2 v_position; // normalized position where x 0..1, y 0..y_ratio
in vec2 v_texcoord;

out vec4 fragColor;

void main() {
    fragColor = texture(tex, vec2(v_texcoord.x, 1.0 - v_texcoord.y));
}