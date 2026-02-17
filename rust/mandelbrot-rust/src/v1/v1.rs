// rendering the mandelbrot set using a single-threaded CPU implementation in Rust

use image::{ImageBuffer, Rgb, RgbImage};
use num_complex::Complex;
use std::time::Instant;
use std::path::Path;

struct Params {
    iterations: u32,
    threshold: f64,
    // range for mandelbrot set
    real_range: (f64, f64),
    imaginary_range: (f64, f64),
    image_dimensions: (u32, u32),
    output: String,
    // two colors to gradient between
    color_1: [u8; 3],
    color_2: [u8; 3],
    color_fill: [u8; 3],
    color_blend: f64, // exponent factor of color easing
}


fn calculate_pixel(c: Complex<f64>, p: &Params) -> [u8; 3] {
    let mut z = Complex::new(0.0, 0.0);
    let mut steps = 0;

    // the main loop for calculating the mandelbrot set
    for i in 0..p.iterations {
        if z.norm() > p.threshold {
            break;
        }
        z = z * z + c;
        steps = i; // tracks the last iteration count
    }

    // if z is still within threshold, it's inside the set (use fill color)
    if z.norm() <= p.threshold {
        return p.color_fill;
    }

    // interpolate color
    // alpha = (steps / iterations) ^ blend
    // steps + 1.0 to smooth it slightly, or strict steps for more contrast
    let alpha = (steps as f64 / p.iterations as f64).powf(p.color_blend);

    // linear interpolation between color_1 and color_2 based on alpha
    let r = (p.color_1[0] as f64 * (1.0 - alpha) + p.color_2[0] as f64 * alpha) as u8;
    let g = (p.color_1[1] as f64 * (1.0 - alpha) + p.color_2[1] as f64 * alpha) as u8;
    let b = (p.color_1[2] as f64 * (1.0 - alpha) + p.color_2[2] as f64 * alpha) as u8;

    [r, g, b]
}


fn main() {
    let start_time = Instant::now(); // for benchmarking time

    // size of image in pixels
    let width = 2000;
    let height = width;

    let p = Params {
        iterations: 200,
        threshold: 2.0,
        real_range: (-1.5, 0.5),
        imaginary_range: (-1.0, 1.0),
        image_dimensions: (width, height),
        output: String::from("output_rust.png"),
        color_1: [0, 0, 0],
        color_2: [255, 0, 0],
        color_fill: [0, 0, 0],
        color_blend: 0.4,
    };

    println!("=========================================================");
    println!("Information:");
    println!("Output Destination: {}", p.output);
    println!("Image Dimensions: {width}px by {height}px");
    println!("Iterations: {}", p.iterations);
    println!("Threshold: {}", p.threshold);
    println!("Real Range: [{}, {}]", p.real_range.0, p.real_range.1);
    println!("Imaginary Range: [{}, {}]", p.imaginary_range.0, p.imaginary_range.1);
    println!("=========================================================");

    // create the image buffer
    let mut img: RgbImage = ImageBuffer::new(width, height);

    // calculate step sizes for mapping pixels to complex coordinates
    let real_step = (p.real_range.1 - p.real_range.0).abs() / width as f64;
    let imaginary_step = (p.imaginary_range.1 - p.imaginary_range.0).abs() / height as f64;

    // main loop
    for y in 0..height {
        // map pixel y to imaginary coordinate
        // top of image is positive imaginary, bottom is negative imaginary
        let imaginary = p.imaginary_range.1  - (y as f64 * imaginary_step) - (imaginary_step / 2.0);

        for x in 0..width {
            // map pixel x to real coordinate
            // left of image is negative real, right is positive real
            let real = p.real_range.0 + (x as f64 * real_step) + (real_step / 2.0);

            let c = Complex::new(real, imaginary);

            let color = calculate_pixel(c, &p);
            let pixel = Rgb(color);
            img.put_pixel(x, y, pixel);
        }
    }

    println!("Calculations done in: {:.2?}", start_time.elapsed());
    img.save(Path::new(&p.output)).expect("Failed to save image");
    println!("Total time taken: {:.2?}", start_time.elapsed());

}
