// Author: tylersfoot
//
// This script generates a mandelbrot fractal as an image.
// To save time, since the mandelbrot set is symmetrical, only half of the image is rendered, and then mirrored later on.


use std::time::Instant;
use rand::prelude::*;
// https://crates.io/crates/image
// https://crates.io/crates/num-complex
// https://crates.io/crates/rand


fn render_image() {
    println!("{}", iterations);
}

fn main() {
    println!("Script started!");

    let now = Instant::now(); // for benchmarking time

    let iterations = 200; // 200
    let threshold = 2; // 2
    // range for mandelbrot set in image
    let real_range: [f64; 2] = [-1.5, 0.5]; // [-1.5, 0.5]
    let mut imaginary_range: [f64; 2] = [-1.0, 1.0]; // [-1, 1]
    // size of image in pixels
    let mut imgx = 64;
    let mut imgy = imgx;
    // two colors to gradient between
    let color_1 = (0, 0, 0);
    let color_2 = (255, 0, 0);
    let color_fill = (0, 0, 0);
    // exponent factor of color easing
    let color_blend = 0.4;
    let bar_length = 100;
    let sections = 4;

    println!("
=========================================================
Information:
Image Dimensions: {imgx}px by {imgy}px
Iterations: {iterations}
Threshold: {threshold}
Real Range: [{real_rangex}, {real_rangey}]
Imaginary Range: [{imaginary_rangex}, {imaginary_rangey}]
=========================================================
    ", imgx=imgx, imgy=imgy, iterations=iterations, threshold=threshold,
    real_rangex=real_range[0], real_rangey=real_range[1],
    imaginary_rangex=imaginary_range[0], imaginary_rangey=imaginary_range[1]);

    let mut rng = rand::thread_rng();

    // cutting some values in half for only rendering half the image
    // imgy = imgx / 2;
    // imaginary_range[0] = 0.0;

    // create a new ImgBuf with width: imgx and height: imgy
    let mut imgbuf = image::ImageBuffer::new(imgx, imgy);

    render_image();

    // iterate over the coordinates and pixels of the image
    for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
        let r = (255.0 * rng.gen::<f64>()).round() as u8;
        let g = (255.0 * rng.gen::<f64>()).round() as u8;
        let b = (255.0 * rng.gen::<f64>()).round() as u8;
        *pixel = image::Rgb([r, g, b]);
    }

    // // A redundant loop to demonstrate reading image data
    // for x in 0..imgx {
    //     for y in 0..imgy {
    //         let cx = y as f32 * scalex - 1.5;
    //         let cy = x as f32 * scaley - 1.5;

    //         let c = num_complex::Complex::new(-0.4, 0.6);
    //         let mut z = num_complex::Complex::new(cx, cy);

    //         let mut i = 0;
    //         while i < 255 && z.norm() <= 2.0 {
    //             z = z * z + c;
    //             i += 1;
    //         }

    //         let pixel = imgbuf.get_pixel_mut(x, y);
    //         let image::Rgb(data) = *pixel;
    //         *pixel = image::Rgb([data[0], i as u8, data[2]]);
    //     }
    // }

    // save the image as “fractal.png”, the format is deduced from the path


    let elapsed = now.elapsed();
    println!("Total time taken: {:.2?}", elapsed);
    imgbuf.save("fractal.png").unwrap();
    println!("Script finished!");
}
