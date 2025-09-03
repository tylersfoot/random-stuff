#![allow(unused)]

use core::f32;
use std::env;
use std::iter::Sum;
use std::path::Path;
use std::process::exit;
use image::ImageReader;
use rand::{prelude::*, random};
use rand::distr::weighted::WeightedIndex;
use std::time::Instant;
use rayon::prelude::*;

type Point = [f32; 3];

fn find_indices(input_points: &[Point], centroids: &[Point]) -> Vec<usize> {
    // finds the index of the closest centroid for each input point
    input_points.par_iter().map(|point| {
        centroids.iter().enumerate()
            .min_by(|(_, a), (_, b)| {
                let dist_a = find_distance_squared(point, a);
                let dist_b = find_distance_squared(point, b);
                dist_a.partial_cmp(&dist_b).unwrap()
            })
            .map(|(index, _)| index)
            .unwrap()
    }).collect()
}

fn step(input_points: &[Point], indices: &[usize], centroids: &[Point]) -> (Vec<Point>, f32) {
    // does one step of the k-means clustering algorithm

    let k = centroids.len();
    // the sums of each centroid's points' positions
    let mut point_sums: Vec<Point> = vec![[0.0, 0.0, 0.0]; k];
    // how many points are in the centroid's cluster
    let mut counts: Vec<usize> = vec![0; k];

    for (i, point) in input_points.iter().enumerate() {
        let cluster_index = indices[i];
        point_sums[cluster_index][0] += point[0];
        point_sums[cluster_index][1] += point[1];
        point_sums[cluster_index][2] += point[2];
        counts[cluster_index] += 1;
    }

    let mut new_centroids: Vec<Point> = Vec::with_capacity(k);
    let mut total_movement = 0.0;
    
    // find the new average position for each centroid
    for i in 0..k {
        let count = counts[i];
        if count > 0 {
            let new_position = [
                point_sums[i][0] / count as f32,
                point_sums[i][1] / count as f32,
                point_sums[i][2] / count as f32
            ];
            total_movement += find_distance(&new_position, &centroids[i]);
            new_centroids.push(new_position);
        } else {
            // cluster empty, keep the old centroid
            new_centroids.push(centroids[i]);
        }
    }

    (new_centroids, total_movement)


    // // the new centroids
    // let mut new_centroids: Vec<Point> = Vec::new();
    // // the average centroid movement
    // let mut movement = 0.0;

    // // loop through each centroid and find its new position
    // for (index, centroid) in centroids.iter().enumerate() {
    //     let mut points_in_cluster = Vec::new();

    //     // list of all input points assigned to this centroid
    //     for i in 0..input_points.len() {
    //         if indices[i] == index {
    //             points_in_cluster.push(input_points[i]);
    //         }
    //     }

    //     // if this cluster isnt empty, find the avg position
    //     if !points_in_cluster.is_empty() {
    //         // find the mean of the x, y, and z coords
    //         let length = points_in_cluster.len() as f32;
    //         let mut totals = [0.0, 0.0, 0.0];

    //         for i in points_in_cluster {
    //             totals[0] += i[0];
    //             totals[1] += i[1];
    //             totals[2] += i[2];
    //         }

    //         let new_position = [
    //             totals[0] / length,
    //             totals[1] / length,
    //             totals[2] / length
    //         ];

    //         new_centroids.push(new_position);
    //         movement += (new_position[0] - centroid[0]).abs() 
    //                   + (new_position[1] - centroid[1]).abs()
    //                   + (new_position[2] - centroid[2]).abs();
    //     } else {
    //         // no points in cluster, keep old position
    //         new_centroids.push(centroids[index]);
    //     }
    // }
    // movement /= centroids.len() as f32;
    // (new_centroids, movement)
}

fn find_distance(p1: &Point, p2: &Point) -> f32 {
    // Euclidean distance formula (3D)
    let dx = p1[0] - p2[0];
    let dy = p1[1] - p2[1];
    let dz = p1[2] - p2[2];

    (dx*dx + dy*dy + dz*dz).powf(0.5)
}

fn find_distance_squared(p1: &Point, p2: &Point) -> f32 {
    // Euclidean distance formula (3D)
    let dx = p1[0] - p2[0];
    let dy = p1[1] - p2[1];
    let dz = p1[2] - p2[2];

    dx*dx + dy*dy + dz*dz
}

fn main() {
    let k = 256; // the amount of final colors

    let start = Instant::now();
    let mut rng = rand::rng();

    let path = Path::new("./images/input4.jpg");
    let image = image::open(path).unwrap().into_rgb8();
    let (w, h) = (image.width(), image.height());
    let px = w * h;
    println!("Image dimensions: ({w}, {h})\nTotal pixels: {px}");

    // list of all the pixel colors in the image (including duplicates)
    // if image is very large, only calculate on a subsample of pixels
    let mut pixels: Vec<Point> = image.pixels()
        .map(|pixel| {
            [pixel[0] as f32, pixel[1] as f32, pixel[2] as f32]
        }).collect();
    let mut input_points: Vec<Point> = Vec::new();

    if px > 100_000 {
        let mut random_idx: Vec<u32> = rng.random::<[u32; 100_000]>().to_vec();
        random_idx.sort();
        random_idx.dedup();
        input_points = random_idx
            .iter()
            .map(|i| pixels[(*i % px) as usize]).collect();
    } else {
        input_points = pixels.clone();
    }
    println!("{}", input_points.len());

    // stores the index of the closest centroid for each input point
    // len(indices) = len(input_points)
    let mut indices: Vec<usize> = Vec::new();

    // initializing centroids (output colors/points) with K-means++
    let mut centroids: Vec<Point> = Vec::new();
    // put first centroid at random input point
    let first_point_index = rng.random_range(0..input_points.len());
    centroids.push(input_points[first_point_index]);
    let mut min_distances: Vec<f32> = vec![f32::INFINITY; input_points.len()]; // each point's minimum distance to centroids
    min_distances[first_point_index] = 0.0;

    // find all other initial centroid positions
    for centroid in 0..k-1 {
        std::io::Write::flush(&mut std::io::stdout()).unwrap();
        print!("Initializing centroids: {}/{k}   \r", centroid+2);

        // find distances of every non-chosen point to the closest centroid
        for (i, point) in input_points.iter().enumerate() {
            let distance = find_distance_squared(point, &centroids[centroid]);
            // overwrite the distance if its smaller
            min_distances[i] = min_distances[i].min(distance);
        }

        // choose new centroid position based on point weights
        let dist = WeightedIndex::new(&min_distances).unwrap();
        let chosen_index = dist.sample(&mut rng);
        min_distances[chosen_index] = 0.0; // make sure this point isn't chosen again later
        centroids.push(input_points[chosen_index]);
    }
    println!();

    let mut movement = f32::INFINITY;
    let max_steps = 100;

    for i in 0..max_steps {
        indices = find_indices(&input_points, &centroids);
        (centroids, movement) = step(&input_points, &indices, &centroids);
        std::io::Write::flush(&mut std::io::stdout()).unwrap();
        print!("Calculating steps: {} - movement: {}    \r", i+1, movement);
        if movement <= 0.1 {
            break;
        }
    }
    println!();

    // generate the output image
    let mut output_image = image::ImageBuffer::new(w, h);
    let full_indices = find_indices(&pixels, &centroids);
    for (i, (x, y, pixel)) in output_image.enumerate_pixels_mut().enumerate() {
        let index = full_indices[i]; // output color index of this pixel
        let color = centroids[index]; // output color
        let color2 = [color[0] as u8, color[1] as u8, color[2] as u8];
        *pixel = image::Rgb(color2);
    }
    output_image.save("./images/output.png").unwrap();

    println!("Done! Took {:?}", start.elapsed());
}