shader_type canvas_item;

uniform sampler2D metaball_map;
uniform sampler2D alternate_universe;
uniform vec2 screen_size;
uniform vec4 outline_color;
uniform vec2 virtual_resolution = vec2(200.0, 100.0);
uniform float density_threshold = 0.001;
uniform float outline_radius;

void fragment() {
    vec2 uv = FRAGCOORD.xy / screen_size;

    // Check the density of the current pixel
    float density = texture(metaball_map, uv).a;

    // Neighborhood alpha accumulator
    float neighbor_alpha = 0.0;

    // Compute pixel size
    float p_w = 1.0 / screen_size.x;
    float p_h = 1.0 / screen_size.y;

    // Iterate through neighbors within the outline radius
    for (int nx = -int(outline_radius); nx <= int(outline_radius); nx++) {
        for (int ny = -int(outline_radius); ny <= int(outline_radius); ny++) {
            // Only consider pixels within the circle
            if (float(nx * nx + ny * ny) <= outline_radius * outline_radius) {
                // Sample the neighboring pixel's density
                float neighbor_density = texture(metaball_map, uv + vec2(float(nx) * p_w, float(ny) * p_h)).a;
                // Accumulate alpha if the neighbor is "inside" the metaball
                neighbor_alpha += step(density_threshold, neighbor_density);
            }
        }
    }

    // Clamp neighbor alpha to [0, 1]
    neighbor_alpha = clamp(neighbor_alpha, 0.0, 1.0);

    // Check if this pixel is part of the outline
    float is_outline = neighbor_alpha * (1.0 - step(density_threshold, density));

    // Discard pixels outside both the metaball interior and the outline
    if (density < density_threshold && is_outline <= 0.0) {
        discard;
    }

    // Sample the alternate universe texture for the metaball interior
    vec4 middle_color = texture(alternate_universe, uv);

    // Blend the interior and the outline colors
    COLOR = mix(middle_color, outline_color, is_outline);
}