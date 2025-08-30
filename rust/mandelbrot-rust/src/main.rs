// Author: tylersfoot
//
// This script generates a mandelbrot fractal as an image.
// To save time, since the mandelbrot set is symmetrical, only half of the image is rendered, and then mirrored later on.


use std::time::Instant;
use rand::prelude::*;
use num::complex::Complex;
// https://crates.io/crates/image
// https://crates.io/crates/num-complex
// https://crates.io/crates/rand


fn linspace(start: f64, end: f64, num: u32) -> Vec<f64> {
    let step = (end - start) / (num as f64 - 1.0);
    (0..num).map(|i| start + i as f64 * step).collect()
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
    let mut imgx = 256;
    let mut imgy = imgx;
    // two colors to gradient between
    let color_1 = (0, 0, 0);
    let color_2 = (255, 0, 0);
    let color_fill = (0, 0, 0);
    // exponent factor of color easing
    let color_blend = 0.4;
    let bar_length = 100;
    let sections = 4;
    let mut percent_done = 0;

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

    // iterate over the coordinates and pixels of the image (TEMPORARY)
    for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
        let r = (255.0 * rng.gen::<f64>()).round() as u8;
        let g = (255.0 * rng.gen::<f64>()).round() as u8;
        let b = (255.0 * rng.gen::<f64>()).round() as u8;
        *pixel = image::Rgb([r, g, b]);
    }
    
        // calculates the real & imaginary coordinates based on the ranges and image dimensions
        let real_step_size = (real_range[0] - real_range[1] / imgx as f64).abs();
        let real_values = linspace(real_range[0] + real_step_size / 2.0, real_range[1] - real_step_size / 2.0, imgx);
        let imaginary_step_size = (imaginary_range[0] - imaginary_range[1] / imgy as f64).abs();
        let imaginary_values = linspace(imaginary_range[1] + imaginary_step_size / 2.0, imaginary_range[0] - imaginary_step_size / 2.0, imgy);
        
        // create a 2D array of complex numbers (each pixel as a complex number)
        let mut c_values = vec![];
        for real in &real_values {
            let mut row = vec![];
            for imaginary in &imaginary_values {
                let c = Complex::new(real, imaginary);
                row.push(c);
            }
            c_values.push(row);
        }

        // calculate the mandelbrot set
        // stores steps taken
        let mut steps = vec![vec![0; imgy as usize]; imgx as usize]; // filled with zeroes, same size as c_values
        // stores the current z values
        let mut z_values = vec![vec![0; imgy as usize]; imgx as usize];
        let mut last_percent = -1;


        let mut mask = vec![vec![false; imgy as usize]; imgx as usize];
        for i in 0..iterations {
            // loop through all pixels
            for x in 0..imgx {
                for y in 0..imgy {
                    // if z (absoluted) is less than threshold:
                    mask[x as usize][y as usize] = (z_values[x as usize][y as usize] as i32).abs() <= threshold; // mask = true
                    if mask[x as usize][y as usize] {
                        z_values[x as usize][y as usize] = (z_values[x as usize][y as usize] as i32).pow(2) + c_values[x as usize][y as usize]; // z = z^2 + c
                        steps[x as usize][y as usize] += 1; // increment steps
                    }
                }
            }

            percent_done = ((i as f64 / iterations as f64) * 100.0).round() as i32;
            //     percent_done = int((i / p['iterations']) * 100)
            //     if percent_done > last_percent:
            //         bar = f"[{'#' * int(p['bar_length'] * i / p['iterations'])}{'-' * (p['bar_length'] - int(p['bar_length'] * i / p['iterations']))}] {percent_done}% Done\r"
            //         print(bar, end='')
            //         last_percent = percent_done
        }

    
        // return np.where(np.abs(z_values) <= p['threshold'], -1, steps)


    //     res = mandelbrot(c_values, p)
    //     print(f'\nCalculations done in: {(time.time() - p['start_time']):.2f} seconds')
        
        // create np array that represents all the pixels
    //     data = np.zeros((w, h, 3), dtype=np.uint8)

        // interpolate colors for other points (accounts for res being -1)
    //     data[:, :] = interpolate_color(p, res)
        
    //     data = np.transpose(data, (1, 0, 2))
        
        // convert color array into image
    //     image = Image.fromarray(data, 'RGB')
                
    //     return image

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
