const PI: f32 = 3.14159265358979323846264338327950288;
const PI_FRAC_3: f32 = 1.04719755119659774615421446109316763;

struct Params {
    width: u32,
    height: u32,
    fractal_type: u32,
    _padding: u32,
    julia_k: vec2<f32>, // only used for Julia set
    iterations: u32,
    threshold: f32,
    real_min: f32,
    real_max: f32,
    imaginary_min: f32,
    imaginary_max: f32,
    // calculated step sizes for mapping pixels to complex coordinates
    real_step: f32,
    imaginary_step: f32,
    color_palette: u32,
    color_fill: u32, // packed RGB as 0xRRGGBB
    palette_iterations: u32,
    smooth_coloring: u32,
}

// output texture (write-only)
@group(0) @binding(0)
var output_texture: texture_storage_2d<rgba8unorm, write>;

// parameter buffer
@group(0) @binding(1)
var<uniform> params: Params;

@compute
@workgroup_size(8, 8, 1) // 64 threads per workgroup (8x8)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    // global_id.x is the unique index of this thread across the entire dispatch
    // we need to convert this 1D index into 2D coordinates (x, y) for our image
    let x = global_id.x;
    let y = global_id.y;

    if (x >= params.width || y >= params.height) { return; }

    // calculate the 1D index into the output buffer based on (x, y)
    let index = y * params.width + x;

    // map pixel y to imaginary coordinate
    let imaginary = params.imaginary_min + (f32(y) * params.imaginary_step) + (params.imaginary_step / 2.0);
    // map pixel x to real coordinate
    let real = params.real_min + (f32(x) * params.real_step) + (params.real_step / 2.0);
    // create complex number c from real and imaginary parts
    let c = vec2<f32>(real, imaginary);

    var z = vec2<f32>(0.0, 0.0);
    // for Julia set, start with z = c and use a constant k
    if (params.fractal_type == 2u) { z = c; }

    var steps: f32 = 0.0;
    let threshold_sq = params.threshold * params.threshold;

    // the main loop for calculating the fractal
    for (var i: u32 = 0u; i < params.iterations; i++) {
        let z_sq = z * z; // precalculate
        let mag_sq = z_sq.x + z_sq.y; // dot(z, z)

        if (mag_sq > threshold_sq) {
            steps = f32(i);
            
            // smooth coloring (renormalization)
            if (params.smooth_coloring == 1u) {
                // use log2 because it's faster on GPUs
                let log_zn = log2(mag_sq) * 0.5;
                let nu = log2(log_zn / log2(params.threshold));
                steps = steps + 1.0 - nu;
            }
            break;
        }

        var z_new = vec2<f32>(0.0, 0.0);

        switch (params.fractal_type) {
            case 0u: { // Mandelbrot
                // z = z^2 + c
                z_new.x = z_sq.x - z_sq.y + c.x;
                z_new.y = 2.0 * z.x * z.y + c.y;
            }
            case 1u: { // Burning Ship
                // z = (|Re(z)| + i*|Im(z)|)^2 + c
                z_new.x = z_sq.x - z_sq.y + c.x;
                z_new.y = 2.0 * abs(z.x) * abs(z.y) + c.y;
            }
            case 2u: { // Julia
                // z = z^2 + k (k is a constant complex number)
                z_new.x = z_sq.x - z_sq.y + params.julia_k.x;
                z_new.y = 2.0 * z.x * z.y + params.julia_k.y;
            }
            case 3u: { // Tricorn
                // z = conjugate(z)^2 + c
                // conjugate means Y is negative; (-y)*(-y) = y*y
                z_new.x = z_sq.x - z_sq.y + c.x;
                z_new.y = -2.0 * z.x * z.y + c.y;
            }
            case 4u: { // Buffalo
                // z = |z^2 - z| + c
                z_new.x = abs(z_sq.x - z_sq.y - z.x) + c.x; // |x^2 - y^2 - x|
                z_new.y = abs(2.0 * z.x * z.y - z.y) + c.y; // |2xy - y|
            }
            case 5u: { // Celtic
                // z = |Re(z^2)| + i*Im(z^2) + c
                z_new.x = abs(z_sq.x - z_sq.y) + c.x;
                z_new.y = 2.0 * z.x * z.y + c.y;
            }
            default: { // Fallback
                z_new = vec2<f32>(0.0, 0.0);
            }
        }
        z = z_new;
    }

    // if z is still within the threshold, it's inside the set (use fill color)
    if (dot(z, z) <= threshold_sq) {
        let r = f32((params.color_fill >> 16u) & 0xFFu) / 255.0;
        let g = f32((params.color_fill >> 8u) & 0xFFu) / 255.0;
        let b = f32(params.color_fill & 0xFFu) / 255.0;
        textureStore(output_texture, vec2<i32>(i32(x), i32(y)), vec4<f32>(r, g, b, 1.0));
        return;
    }

    // pixel is outside the set, color will be interpolated
    // normalize steps to [0, 1] range
    let normalized_steps = steps / f32(params.palette_iterations);
    let color = gradient(normalized_steps, params.color_palette);
    let coordinates = vec2<i32>(i32(x), i32(y));
    textureStore(output_texture, coordinates, vec4<f32>(color, 1.0));
}

fn gradient(steps: f32, palette: u32) -> vec3<f32> {
    switch (palette) {
        case 0u: {
            // sinebow cyclical gradient
            // https://docs.rs/colorous/latest/src/colorous/cyclical.rs.html#37
            // https://observablehq.com/@mbostock/sinebow

            let t = 0.5 - steps;
            let offsets = vec3<f32>(0.0, 1.0 / 3.0, 2.0 / 3.0);
            let sines = sin(PI * (t + offsets));
            return sines * sines;
        }
        case 1u: {
            let ts = abs(steps - 0.5);
            let h = 360.0 * steps - 100.0;
            let s = 1.5 - 1.5 * ts;
            let l = 0.8 - 0.9 * ts;

            return cubehelix_to_rgb(h, s, l);
        }
        default: {
            return vec3<f32>(0.0, 0.0, 0.0);
        }
    }
}


fn cubehelix_to_rgb(h: f32, s: f32, l: f32) -> vec3<f32> {
    let angle = radians(h);
    let amp = s * l * (1.0 - l);
    
    let cos_h = cos(angle);
    let sin_h = sin(angle);

    let r = l + amp * (-0.14861 * cos_h + 1.78277 * sin_h);
    let g = l + amp * (-0.29227 * cos_h - 0.90649 * sin_h);
    let b = l + amp * (1.97294 * cos_h);

    // Clamp results to 0.0 - 1.0 range
    return saturate(vec3<f32>(r, g, b)); 
}