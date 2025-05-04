
use std::fs::create_dir_all;
use std::io::ErrorKind;
use std::io::{self, Write};
use std::error::Error;
use video_rs::decode::Decoder;
use image::{ImageBuffer, Rgb};
use tokio::task;
use inquire::{
    Text, CustomUserError, Text,
    validator::{StringValidator, Validation},
    autocompletion::{Autocomplete, Replacement}
};
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher;


fn ask_video_path() {
    let validator = |input: &str| if input.chars().count() > 140 {
        Ok(Validation::Invalid("You're only allowed 140 characters.".into()))
    } else {
        Ok(Validation::Valid)
    };

    let status = Text::new("What are you thinking about?")
    .with_validator(validator)
    .prompt();

    match status {
        Ok(status) => println!("Your status: {}", status),
        Err(err) => println!("Error while publishing your status: {}", err),
    }
}


fn clear_temp() {
    let temp_folder = std::env::temp_dir().join("frame-merge");
    if temp_folder.exists() {
        std::fs::remove_dir_all(&temp_folder).expect("failed to remove temp folder");
    }
    create_dir_all(&temp_folder).expect("failed to create temp folder");
}

#[derive(Clone, Default)]
pub struct FilePathCompleter {
    input: String,
    paths: Vec<String>,
}


impl FilePathCompleter {
    fn update_input(&mut self, input: &str) -> Result<(), CustomUserError> {
        if input == self.input && !self.paths.is_empty() {
            return Ok(());
        }

        self.input = input.to_owned();
        self.paths.clear();

        let input_path = std::path::PathBuf::from(input);

        let fallback_parent = input_path
            .parent()
            .map(|p| {
                if p.to_string_lossy() == "" {
                    std::path::PathBuf::from(".")
                } else {
                    p.to_owned()
                }
            })
            .unwrap_or_else(|| std::path::PathBuf::from("."));

        let scan_dir = if input.ends_with('/') {
            input_path
        } else {
            fallback_parent.clone()
        };

        let entries = match std::fs::read_dir(scan_dir) {
            Ok(read_dir) => Ok(read_dir),
            Err(err) if err.kind() == ErrorKind::NotFound => std::fs::read_dir(fallback_parent),
            Err(err) => Err(err),
        }?
        .collect::<Result<Vec<_>, _>>()?;

        for entry in entries {
            let path = entry.path();
            let path_str = if path.is_dir() {
                format!("{}/", path.to_string_lossy())
            } else {
                path.to_string_lossy().to_string()
            };

            self.paths.push(path_str);
        }

        Ok(())
    }

    fn fuzzy_sort(&self, input: &str) -> Vec<(String, i64)> {
        let mut matches: Vec<(String, i64)> = self
            .paths
            .iter()
            .filter_map(|path| {
                SkimMatcherV2::default()
                    .smart_case()
                    .fuzzy_match(path, input)
                    .map(|score| (path.clone(), score))
            })
            .collect();

        matches.sort_by(|a, b| b.1.cmp(&a.1));
        matches
    }
}

impl Autocomplete for FilePathCompleter {
    fn get_suggestions(&mut self, input: &str) -> Result<Vec<String>, CustomUserError> {
        self.update_input(input)?;

        let matches = self.fuzzy_sort(input);
        Ok(matches.into_iter().take(15).map(|(path, _)| path).collect())
    }

    fn get_completion(
        &mut self,
        input: &str,
        highlighted_suggestion: Option<String>,
    ) -> Result<Replacement, CustomUserError> {
        self.update_input(input)?;

        Ok(if let Some(suggestion) = highlighted_suggestion {
            Replacement::Some(suggestion)
        } else {
            let matches = self.fuzzy_sort(input);
            matches
                .first()
                .map(|(path, _)| Replacement::Some(path.clone()))
                .unwrap_or(Replacement::None)
        })
    }
}



#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>>  {
    clear_temp();

    ask_video_path();



    video_rs::init().unwrap();

    let start_time = std::time::Instant::now();
    let mut decoder = Decoder::new(std::env::current_dir().unwrap().join("video.mp4")).unwrap();

    let temp_folder = std::env::temp_dir().join("frame-merge");

    let (width, height) = decoder.size();
    let frame_rate = decoder.frame_rate(); // Assuming 30 FPS if not available

    let max_duration = 2.0; // Max duration in seconds
    // let max_frames = (frame_rate * max_duration).ceil() as usize;

    let mut frame_count = 0;
    let mut elapsed_time = 0.0;
    let mut tasks = vec![];

    for frame in decoder.decode_iter() {
        if let Ok((_timestamp, frame)) = frame {
            if elapsed_time > max_duration {
                break;
            }

            let rgb = frame.slice(ndarray::s![.., .., 0..3]).to_slice().unwrap();

            let img: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::from_raw(width, height, rgb.to_vec())
            .expect("failed to create image buffer");

            let frame_path = format!("{}/frame_{:05}.png", temp_folder.display(), frame_count);

            let task = task::spawn_blocking(move || {
                img.save(&frame_path).expect("failed to save frame");
            });

            tasks.push(task);

            frame_count += 1;
            elapsed_time += 1.0 / frame_rate;

            print!("Saved frame {} at time {:.2} seconds\r", frame_count, elapsed_time);
            io::stdout().flush().unwrap();
        } else {
            break;
        }
    }


    // await all tasks to finish
    for task in tasks {
        task.await.expect("task failed");
    }

    println!("Saved {} frames in the '{}' directory ({}s)", frame_count, temp_folder.display(), start_time.elapsed().as_secs_f32());

    clear_temp();
    Ok(())
}
