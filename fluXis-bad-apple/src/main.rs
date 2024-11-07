use std::process::{Command};
use std::fs;
use std::path::Path;
use std::time::Instant;
use image::{DynamicImage, GenericImageView, Pixel, imageops::FilterType};

fn process_img(image: &DynamicImage, width: u32, height: u32) -> Vec<Vec<u8>> {
    // resize the image to the target dimensions
    let resized = image.resize_exact(width, height, FilterType::Nearest);

    // initialize a 2D vector to store the binarized values
    let mut binarized = vec![vec![0; height as usize]; width as usize];

    // iterate over each pixel in the resized image
    for (x, y, pixel) in resized.pixels() {
        // convert the pixel to grayscale to get brightness
        let grayscale = pixel.to_luma();
        let brightness = grayscale[0];

        // set 1 if brightness is greater than 50, otherwise 0
        binarized[x as usize][y as usize] = if brightness > 50 { 1 } else { 0 };
    }

    binarized
}

fn calculate_notes_sv(frames: Vec<Vec<Vec<u8>>>, width: u32, height: u32, fps: u32) -> (Vec<String>, Vec<String>) {
    // some variables
    let mut notes = Vec::new(); // array of strings in the exact format: {"time":180.0,"lane":1,"holdtime":0.0,"hitsound":":normal","type":0},
    let mut svs = Vec::new(); // array of strings in the exact format: {"time":0,"multiplier":0},
    let offset_ms = 180.0; // offset for song
    let frame_diff_ms = 1000.0 / fps as f64 * 5.0; // ms between each frame
    let row_diff_ms = 0.001; // ms between each row of notes // 0.01 -> 4000x sv (when sv_diff_ms = 1)
    let sv_diff_ms = 0.1; // ms between each sv change (between the fast and the 0)
    let sv_multiplier = row_diff_ms * 50000000.0; // the fast sv

    
    let note_pt1 = String::from(r##"{"time":"##); // + time
    let note_pt2 = String::from(r##","lane":"##); // + lane
    let note_pt3 = String::from(r##","holdtime":0.0,"hitsound":":normal","type":0},"##);
    
    let sv_pt1 = String::from(r##"{"time":"##); // + time
    let sv_pt2 = String::from(r##","multiplier":"##); // + multiplier
    let sv_pt3 = String::from(r##"},"##);
    
    // loop through frames and add notes
    for (frame_num, frame) in frames.iter().enumerate() {
        // loop through each pixel in the frame in order:
        // [1, 2]
        // [3, 4]
        for y in 0..height { // y = row, starting from top of frame
            for x in 0..width { // x = column (lane)
                if frame[x as usize][y as usize] == 1 { // if pixel is white
                    // note construction
                    let note_time = offset_ms + (frame_num as f64 * frame_diff_ms) + ((height - 1 - y) as f64 * row_diff_ms);
                    let note_lane = x + 1;
                    let note = format!("{}{}{}{}{}", note_pt1, note_time, note_pt2, note_lane, note_pt3);
                    notes.push(note);
                }
            }
        }

        // sv construction (once per frame)
        let sv1_time = offset_ms + (frame_num as f64 * frame_diff_ms);
        let sv2_time = sv1_time + sv_diff_ms;
        let sv1 = format!("{}{}{}{}{}", sv_pt1, sv1_time, sv_pt2, sv_multiplier, sv_pt3); // should be right on beat
        let sv2 = format!("{}{}{}{}{}", sv_pt1, sv2_time, sv_pt2, 0, sv_pt3); // should be right after
        svs.push(sv1);
        svs.push(sv2);
    }

    (notes, svs)
}

fn main() {
    println!("Starting script!");
    let mut now = Instant::now();
    let video_path = "./data/badapplevideo.mp4";
    let frames_path = "./data/frames/";
    let fps = 30;
    let width = 10; // 10 keys wide
    let height = 10;
    let mut frames: Vec<Vec<Vec<u8>>> = Vec::new(); // raw data of all frames

    // create the frames output directory if it doesn't exist
    if !Path::new(frames_path).exists() {
        fs::create_dir_all(frames_path).expect("Failed to create frames directory");
    } else {
        // clear existing frames in the output directory
        fs::read_dir(frames_path)
            .expect("Failed to read frames directory")
            .for_each(|entry| {
                let entry = entry.expect("Failed to read entry");
                if entry.path().is_file() {
                    fs::remove_file(entry.path()).expect("Failed to delete frame file");
                }
            });
        println!("Cleared old frames from {}", frames_path);
    }

    // run ffmpeg command to extract frames
    println!("Extracting frames from video...");
    let ffmpeg_output = Command::new("ffmpeg")
        .args(&[
            "-i", video_path, // input video path
            "-vf", &format!("fps={}", fps), // set new FPS
            &format!("{}/%d.png", frames_path), // output format
        ])
        .output()
        .expect("Failed to execute ffmpeg");

    if ffmpeg_output.status.success() {
        println!("Frames successfully extracted to {} in {:.2?}", frames_path, now.elapsed());
    } else {
        eprintln!("ffmpeg error: {}", String::from_utf8_lossy(&ffmpeg_output.stderr));
    }
    
    // loop through frames and add data to array
    println!("Processing frames...");
    now = Instant::now();

    let frame_count = fs::read_dir(frames_path)
        .expect("Failed to read directory")
        .filter_map(Result::ok)  // Filter out any errors
        .filter(|entry| entry.path().is_file()) // Only count files, ignore subdirectories
        .count();

    for i in 0..frame_count {
    // for i in 0..120 {
        frames.push(process_img(&image::open(&format!("./data/frames/{}.png", i+1)).expect("Failed to open image"), width, height));
    }
    
    println!("Processed {} frames in {:.2?}", frames.len(), now.elapsed());

    // TEST PRINT FRAME
    // let data = process_img(&image::open("./data/frames/174.png").expect("Failed to open image"), width, height);
    // for y in 0..height {
    //     for x in 0..width {
    //         let symbol = if data[x as usize][y as usize] == 1 { '█' } else { '░' };
    //         print!("{} ", symbol);
    //     }
    //     println!();
    // }
    
    println!("Converting frames to notes and generating map file...");
    now = Instant::now();
    
    let map_pt1 = String::from(r##"{"AudioFile":"badappleaudio.mp3","BackgroundFile":"badapplebg.jpg","CoverFile":"badapplesquare.png","VideoFile":"","EffectFile":"","StoryboardFile":"","metadata":{"title":"Bad Apple!!","artist":"Masayoshi Minoshima ft. nomico","mapper":"tylersfoot","difficulty":"Rotten Apple","source":"","bg-source":"","cover-source":"","tags":"","previewtime":-1},"colors":{"accent":"#000000","primary":"#000000","secondary":"#000000","middle":"#000000"},"##);
    let map_pt2 = String::from(r##""TimingPoints":[{"time":180,"bpm":60,"signature":4,"hide-lines":true}],"##);
    let map_pt3 = String::from(r##","HitSoundFades":[],"AccuracyDifficulty":2.0,"HealthDifficulty":2.0}"##);
    
    let notes; // array of strings in the exact format: {"time":180.0,"lane":1,"holdtime":0.0,"hitsound":":normal","type":0},
    let svs; // array of strings in the exact format: {"time":0,"multiplier":0},
    (notes, svs) = calculate_notes_sv(frames, width, height, fps); // calculate notes and svs strings

    // loop through notes and add to hitobjects array
    let mut map_hitobjects = String::from(r##""HitObjects":["##); // the hitobjects part of the map file
    let mut notecount = 0;
    for note in notes {
        map_hitobjects.push_str(&*note);
        notecount += 1;
    }
    map_hitobjects.pop(); // remove trailing comma
    map_hitobjects.push_str("],");

    // loop through svs and add to scroll velocities array
    let mut map_sv = String::from(r##""ScrollVelocities":["##); // the scroll velocities part of the map file
    for sv in svs {
        map_sv.push_str(&*sv);
    }
    map_sv.pop(); // remove trailing comma
    map_sv.push_str("]");
    
    let map_string = format!("{}{}{}{}{}", map_pt1, map_hitobjects, map_pt2, map_sv, map_pt3);
    
    // write the final map file
    let map_path = r##"./data/Masayoshi Minoshima ft. nomico - Bad Apple!! (tylersfoot) [Rotten Apple].osu.fsc"##;
    if Path::new(map_path).exists() {
        fs::remove_file(map_path).expect("Failed to delete old map file");
    }
    fs::write(map_path, map_string).expect("Failed to write map file");

    println!("Generated map file ({} notes) in {:.2?}", notecount, now.elapsed());
}
