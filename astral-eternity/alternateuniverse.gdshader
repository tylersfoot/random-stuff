shader_type canvas_item;

uniform float animation_time;
uniform vec2 parallax; // x and y movement
uniform vec3 colors[16];
uniform vec2 canvas_size;
uniform vec2 screen_size;
uniform float scale; // 1.0 is normal

const float PI2 = 6.283185307179586476925286766559;

uniform sampler2D end_texture : filter_nearest; 

mat2 mat2_rotate_z(float rad) {
    // Returns the rotation matrix for a vec2 around the z-axis
    rad = mod(rad, PI2);
    return mat2(
        vec2(cos(rad), -sin(rad)),
        vec2(sin(rad), cos(rad))
    );
}

vec2 end_portal_layer(float layer, vec2 uv) {
    // Scale and resize to prevent stretching
    uv *= canvas_size * (0.00070 / scale);

    // Resize based on screen size (to standardize for 4k vs 1080p)
    if (screen_size.x / screen_size.y > 1.0) {
        uv *= 1080.0 / screen_size.x;
    } else {
        uv *= 1920.0 / screen_size.y;
    }

    // Apply parallax effect
    uv -= parallax * pow(layer, 5.0) / 759375000.0;

    // Apply layer-specific scaling and rotation
    float layer_scale = (4.5 - layer / 4.0) * 2.0;
    float rotation = radians((layer * layer * 4321.0 + layer * 9.0) * 2.0);
    uv = mat2_rotate_z(rotation) * uv * layer_scale;

    // Apply layer-specific translation based on animation time
    uv.x += (17.0 / layer + 0.25);
    uv.y += ((2.0 + layer / 1.5) * (animation_time * 1.5) + 0.25);

    // Loop the texture coordinates
    uv = mod(uv, 1.0);

    return uv;
}

void fragment() {
    vec3 color = vec3(0.0);
    for (int i = 0; i < 15; i++) {
        vec2 tex_coords = end_portal_layer(float(i + 1), UV);
        color += texture(end_texture, tex_coords).rgb * colors[int(mod(float(i), 16.0))];
    }
    COLOR = vec4(color, 1.0);
}
