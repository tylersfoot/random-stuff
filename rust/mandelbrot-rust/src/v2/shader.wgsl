struct Params {
    width: u32,
    height: u32,
    iterations: u32,
    threshold: f32,
    real_min: f32,
    real_max: f32,
    imaginary_min: f32,
    imaginary_max: f32,
    // packed RGB as 0xRRGGBB
    color_1: u32,
    color_2: u32,
    color_fill: u32,
    color_blend: f32,
    // calculated step sizes for mapping pixels to complex coordinates
    real_step: f32,
    imaginary_step: f32,
}

// output buffer (RGBA8 format, packed as u32) for the image pixels
@group(0) @binding(0)
var<storage, read_write> output: array<u32>;

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
    // calculate the 1D index into the output buffer based on (x, y)
    let index = y * params.width + x;

    if (x >= params.width || y >= params.height) {
        return; // out of bounds, do nothing
    }

    // map pixel y to imaginary coordinate
    let imaginary = params.imaginary_min + (f32(y) * params.imaginary_step) + (params.imaginary_step / 2.0);

    // map pixel x to real coordinate
    let real = params.real_min + (f32(x) * params.real_step) + (params.real_step / 2.0);

    // create complex number c from real and imaginary parts
    let c = vec2<f32>(real, imaginary);

    var z = vec2<f32>(0.0, 0.0);
    var steps: u32 = 0;

    let threshold_squared = params.threshold * params.threshold;

    // the main loop for calculating the mandelbrot set
    for (var i: u32 = 0; i < params.iterations; i = i + 1) {
        if (dot(z, z) > threshold_squared) {
            break;
        }

        // z = z^2 + c
        let z_new = vec2<f32>(
            (z.x * z.x) - (z.y * z.y) + c.x,
            (2.0 * z.x * z.y) + c.y
        );
        z = z_new;
        steps = i; // tracks the last iteration count
    }

    // if z is still within threshold, it's inside the set (use fill color)
    if (dot(z, z) <= threshold_squared) {
        output[index] = params.color_fill | (255u << 24u); // set alpha to 255
        return;
    }

    // interpolate color
    // alpha = (steps / iterations) ^ blend
    let alpha = pow((f32(steps) / f32(params.iterations)), params.color_blend);

    // linear interpolation between color_1 and color_2 based on alpha
    let r1 = f32((params.color_1 >> 16u) & 0xFFu);
    let g1 = f32((params.color_1 >> 8u) & 0xFFu);
    let b1 = f32(params.color_1 & 0xFFu);
    let r2 = f32((params.color_2 >> 16u) & 0xFFu);
    let g2 = f32((params.color_2 >> 8u) & 0xFFu);
    let b2 = f32(params.color_2 & 0xFFu);

    let r = r1 * (1.0 - alpha) + r2 * alpha;
    let g = g1 * (1.0 - alpha) + g2 * alpha;
    let b = b1 * (1.0 - alpha) + b2 * alpha;

    // pack the interpolated color back into a u32 (0xRRGGBBAA format, with alpha fixed at 255)
    let color = u32(r) | (u32(g) << 8u) | (u32(b) << 16u) | (255u << 24u);
    output[index] = color;
}
