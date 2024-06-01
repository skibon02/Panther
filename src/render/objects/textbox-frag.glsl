#version 300 es
precision highp float;

uniform sampler2D tex;
uniform int u_style;

in vec2 v_position;
in vec2 v_texcoord;

out vec4 fragColor;

void main() {

    float intencity = texture(tex, v_texcoord).r;
    if (intencity > 0.01) {
        //cute rainbow based on position
        if (u_style == 0) {
            vec3 color = vec3(0.5 + 0.5 * sin(v_position.x), 0.5 + 0.5 * sin(v_position.y - 0.7), 0.5 + 0.5 * sin(v_position.x + v_position.y));
            fragColor = vec4(color, intencity);
        }
        // white color
        if (u_style == 1) {
            vec3 color = vec3(1.0, 0.9, 1.0);
            fragColor = vec4(color, intencity);
        }

        // red color
        if (u_style == 2) {
            vec3 color = vec3(1.0, 0.1, 0.2);
            fragColor = vec4(color, intencity);
        }
    }
    else {
        discard;
    }
}