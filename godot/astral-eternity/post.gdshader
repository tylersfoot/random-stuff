shader_type canvas_item;

uniform sampler2D SCREEN_TEXTURE : hint_screen_texture, filter_nearest;

// Parameters for customization
uniform float glow_threshold : hint_range(0.0, 1.0) = 0.0; // Brightness threshold for glowing
uniform float glow_intensity : hint_range(0.0, 2.0) = 0.0; // Intensity of the glow
uniform float glow_radius : hint_range(1.0, 5.0) = 1.0; // Glow radius in pixel size

vec4 sample_glow_pixel(sampler2D tex, vec2 uv, vec2 offset) {
    vec4 color = textureLod(tex, uv + offset, 0.0);
    return max(color - vec4(glow_threshold), vec4(0.0)); // Extract bright areas
}

void fragment() {
    vec2 ps = SCREEN_PIXEL_SIZE * glow_radius; // Adjust sampling by glow radius

    // Sample pixels in a cross pattern
    vec4 glow_color = vec4(0.0);
    glow_color += sample_glow_pixel(SCREEN_TEXTURE, SCREEN_UV, vec2(-ps.x, 0.0));
    glow_color += sample_glow_pixel(SCREEN_TEXTURE, SCREEN_UV, vec2(ps.x, 0.0));
    glow_color += sample_glow_pixel(SCREEN_TEXTURE, SCREEN_UV, vec2(0.0, -ps.y));
    glow_color += sample_glow_pixel(SCREEN_TEXTURE, SCREEN_UV, vec2(0.0, ps.y));

    // Blend the glow with the original color
    vec4 original_color = texture(SCREEN_TEXTURE, SCREEN_UV);
    vec4 final_color = original_color + glow_color * glow_intensity;

    COLOR = vec4(final_color.rgb, original_color.a); // Maintain original alpha
}
