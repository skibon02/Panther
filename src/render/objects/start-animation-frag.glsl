#version 300 es
precision highp float;

uniform float y_ratio;
uniform vec4 t;
uniform sampler2D tex;

in vec2 v_position; // normalized position where x 0..1, y 0..y_ratio
in vec2 v_texcoord;

out vec4 fragColor;

bool is_in_rect(float t) {

    float side_offs = 0.45;
    float top_offs = 0.0;

    side_offs -= 0.15 * clamp(t, 0.0, 1.0);

    if (t >= 1.0) {
        float d_t = clamp(t - 1.0, 0.0, 1.0);
        top_offs += 0.25 * d_t;
        side_offs -= 0.05 * d_t;
    }

    if (t >= 2.0) {
        float d_t = clamp(t - 2.0, 0.0, 1.0);
        top_offs += 0.05 * d_t;
        side_offs -= 0.05 * d_t;
    }

    float initial_sickness = 0.015;
    float bot_side = 1.0 + initial_sickness;
    float top_side = bot_side + top_offs;

    vec2 pt = vec2(side_offs, bot_side);
    if (distance(pt, v_position) < initial_sickness) {
        return true;
    }

    pt = vec2(1.0 - side_offs, bot_side);
    if (distance(pt, v_position) < initial_sickness) {
        return true;
    }

    pt = vec2(side_offs, top_side);
    if (distance(pt, v_position) < initial_sickness) {
        return true;
    }

    pt = vec2(1.0 - side_offs, top_side);
    if (distance(pt, v_position) < initial_sickness) {
        return true;
    }

    //rect1
    if (v_position.x > side_offs - initial_sickness && v_position.x < 1.0 - side_offs + initial_sickness
     && v_position.y > bot_side && v_position.y < top_side) {
        return true;
    }

    //rect2
    if (v_position.y > bot_side - initial_sickness && v_position.y < top_side + initial_sickness
     && v_position.x > side_offs && v_position.x < 1.0 - side_offs) {
        return true;
    }

    return false;
}

void main() {
    fragColor = vec4(0.0, 0.0, 0.0, 0.0);

    vec4 color = vec4(0.5, 0.2, 0.9, 1.0);
    if(is_in_rect(clamp(t.r, 0.0, 3.0))) {
        fragColor = color;
    }

    color = vec4(0.4, 0.5, 0.9, 1.0);
    if(is_in_rect(clamp(t.g, 0.0, 3.0))) {
        fragColor = color;
    }

    color = vec4(0.6, 0.8, 0.2, 1.0);
    if(is_in_rect(clamp(t.b, 0.0, 3.0))) {
        fragColor = color;
    }

    color = vec4(1.0, 0.85, 1.0, 1.0);
    if(is_in_rect(clamp(t.a, 0.0, 3.0))) {
        fragColor = color;

        //render gif
        vec2 gifMin = vec2(0.28, 0.95);
        vec2 gifMax = vec2(0.72, 1.35);
        vec2 texcoord = (v_position - gifMin) / (gifMax - gifMin);
        texcoord.y = 1.0 - texcoord.y;
        if (texcoord.y < 0.74) {
            if (v_position.x > gifMin.x && v_position.x < gifMax.x
            && v_position.y > gifMin.y && v_position.y < gifMax.y) {
                float gif_alpha = texture(tex, texcoord).a;
                if (gif_alpha > 0.5) {
                    fragColor = vec4(0.04, 0.02, 0.1, gif_alpha);
                }
            }
        }
    }


}
