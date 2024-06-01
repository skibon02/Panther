#version 300 es
precision highp float;

uniform vec3 color;
uniform float y_ratio;
uniform float u_tab_offset;
uniform float u_top_side;

in vec2 v_position; // normalized position where x 0..1, y 0..y_ratio
in vec2 v_texcoord;

out vec4 fragColor;

void main() {
    fragColor = vec4(0.0, 0.0, 0.0, 0.0);
    float border_radius = 0.05;
    float tab_height = 0.1;
    if (v_position.y > u_top_side - tab_height) {
        float tab_width = 0.25;
        if (v_position.x > u_tab_offset && v_position.x < u_tab_offset + tab_width) {
            fragColor = vec4(color, 1.0);


            if (v_position.y > u_top_side - border_radius) {
                if (v_position.x < u_tab_offset + border_radius) {
                    if (distance(vec2(u_tab_offset + border_radius, u_top_side - border_radius), v_position) > border_radius) {
                        fragColor = vec4(0.0, 0.0, 0.0, 0.0);
                    }
                }
                if (v_position.x > u_tab_offset + tab_width - border_radius) {
                    if (distance(vec2(u_tab_offset + tab_width - border_radius, u_top_side - border_radius), v_position) > border_radius) {
                        fragColor = vec4(0.0, 0.0, 0.0, 0.0);
                    }
                }
            }
        }
    }
    else {
        fragColor = vec4(color, 1.0);
    }
}
