#version 300 es
precision highp float;

uniform float y_ratio;
uniform sampler2D tex;
uniform vec3 u_color;

in vec2 v_position; // normalized position where x 0..1, y 0..y_ratio
in vec2 v_texcoord;

out vec4 fragColor;

void main() {
    fragColor = vec4(u_color, texture(tex, vec2(v_texcoord.x, 1.0 - v_texcoord.y)).a);
}