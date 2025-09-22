use std::{
    collections::HashMap,
    thread,
    time::{
        Duration,
        Instant
    },
    fs::File,
};
use rustfft::{
    FftPlanner,
    num_complex::Complex,
};
use rodio::Decoder;


fn wait(millis: u64) {
    thread::sleep(Duration::from_millis(millis));
}

fn toggle_alt_terminal(enabled: bool) {
    match enabled {
        true  => { print!("\x1b[?1049h"); }
        false => { print!("\x1b[?1049l"); }
    }
}

fn perform_fft(audio_samples: &[i32]) -> Vec<f32> {
    let fft_size = audio_samples.len();

    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(fft_size);

    let mut buffer: Vec<Complex<f32>> = audio_samples
        .iter()
        .map(|&sample| Complex::new(sample as f32, 0.0))
        .collect();

    fft.process(&mut buffer);

    // take only the first half of bins (due to Nyquist theorem)
    let useful_bins = fft_size / 2;
    buffer
        .iter()
        .take(useful_bins)
        .map(|bin| bin.norm())
        .collect()
}

fn main() -> Result<(), hound::Error> {
    let path = "orbital.wav";

    // -------- rodio playback
    let stream_handle = rodio::OutputStreamBuilder::open_default_stream()
        .expect("open default audio stream");
    let sink = rodio::Sink::connect_new(stream_handle.mixer());
    let file = File::open(path).expect("Failed to open audio file");
    let source = Decoder::try_from(file).unwrap();
    sink.pause();
    sink.append(source);
    sink.set_volume(1.0);

    // -------- hound wav reading
    let mut reader = hound::WavReader::open(path)?;
    // audio specification (channels, sample rate, bits per sample, sample format)
    let spec = reader.spec();
    println!("Audio Spec: {spec:?}");
    let sample_rate = reader.spec().sample_rate;
    // read the audio samples; type depends on bit depth (e.g., i16, i24, i32, f32)
    let stereo_samples: Vec<i32> = reader.samples::<i32>().filter_map(Result::ok).collect();
    println!("Successfully read {} samples.", stereo_samples.len());

    // only take one channel for visualizing
    let samples: Vec<i32> = stereo_samples.iter().step_by(2).cloned().collect();

    wait(1000);

    let w_waveform = 420;
    let w_eq = 420;
    let h_waveform = 40;
    let h_eq = 30;
    let mut waveform_grid = HashMap::new();
    let mut eq_grid = HashMap::new();

    // find the max amplitude for normalization
    let max = samples.iter().map(|s| s.abs()).max().unwrap_or(i32::MAX);

    toggle_alt_terminal(true);
    print!("\x1b[?25l"); // disable cursor

    let start_time = Instant::now();
    let mut current_time;
    let mut previous_time = 0;
    let end_time = 180 * 1000; // what time to end (in sec)
    let target_fps = 165; // fps to render at
    let render_delay = 1_000 / target_fps; // pass this time to render next frame
    let mut time_buffer = 0; // time elapsed, for delaying rendering

    // EQ setup
    let fft_size = 2048; // how many samples to analyze for the equalizer
    // frequency increase per fft bin
    let frequency_per_bin = sample_rate as f32 / fft_size as f32;
    let mut bar_bin_boundaries = vec![0; w_eq + 1];
    let min_freq: f32 = 20.0; // min frequency to display
    let max_freq: f32 = 20000.0; // max frequency

    // horizontal log scaling
    for i in 0..=w_eq {
        let freq_boundary = min_freq * (max_freq / min_freq).powf(i as f32 / w_eq as f32);
        let bin_index = (freq_boundary / frequency_per_bin).floor() as usize;
        bar_bin_boundaries[i] = bin_index.min(fft_size / 2); // cap at max bin index
    }
    
    // EQ smoothing
    let mut eq_bar_heights: Vec<f32> = vec![0.0; w_eq];
    let falloff_speed = 0.95; // 5% falloff per frame

    let delay = 200; // ms visual offset, increase if visuals are early

    sink.play();

    loop {
        // current_time = start_time.elapsed().as_millis();
        current_time = sink.get_pos().as_millis();
        if current_time >= end_time {
            break;
        }

        time_buffer += current_time - previous_time;
        previous_time = current_time;

        if time_buffer < render_delay {
            // not time to render the next frame yet
            wait(1);
            continue;
        }
        time_buffer -= render_delay;

        let sample_offset = (current_time.saturating_sub(delay) as usize * sample_rate as usize) / 1000;

        // dont read past the end of the samples
        if sample_offset + w_waveform.max(fft_size) >= samples.len() {
            continue;
        }

        print!("\x1b[H"); // cursor position to top left

        // clear grid
        for y in 0..h_waveform {
            for x in 0..w_waveform {
                waveform_grid.insert((x as i16, y as i16), " ");
            }
        }
        for y in 0..h_eq {
            for x in 0..w_eq {
                eq_grid.insert((x as i16, y as i16), " ");
            }
        } 

        // waveform calculation
        // for (i, sample) in (samples[sample_offset..(sample_offset + w)]).iter().enumerate() {
        //     let mut height = *sample as f32 / max as f32; // -1 to 1
        //     height = height / 2.0 + 0.5; // 0 to 1
        //     height *= (h - 1) as f32; // 0 to h
        //     height = height.round();

        //     let mid = (0.5 * (h - 1) as f32).round();
        //     if height > mid {
        //         for y in 0..=(height - mid) as usize {
        //             waveform_grid.insert((i as i16, (mid as usize + y) as i16), "█");
        //         }
        //     } else if height < mid {
        //         for y in 0..=(mid - height) as usize {
        //             waveform_grid.insert((i as i16, (mid as usize - y) as i16), "█");
        //         }
        //     } else {
        //         waveform_grid.insert((i as i16, mid as i16), "█");
        //     }

        //     // waveform_grid.insert((i as i16, height.round() as i16), "█");
        // }

        // w is the width of the display in character columns.
        // We will now process 2 * w samples to fit two samples per column.

        // Helper closure to calculate the vertical position for a sample.
        // This avoids code duplication and captures `max` and `h` from the outer scope.
        let calculate_height = |sample: i32| -> f32 {
            let mut height = sample as f32 / max as f32; // -1.0 to 1.0
            height = height / 2.0 + 0.5; // 0.0 to 1.0
            height * h_waveform as f32 // 0.0 to h-1
        };

        // For an even height, there are two "middle" rows. We define the boundary here.
        // The wave will go up from the upper-middle row, or down from the lower-middle row.
        let mid_upper = (h_waveform / 2) as i16;
        let mid_lower = (h_waveform / 2 - 1) as i16;

        // This array maps a 4-bit quadrant mask to a specific Unicode character.
        // The bits represent: 8(▘), 4(▝), 2(▖), 1(▗)
        // For example, a mask of 12 (binary 1100) means the upper-left (8) and
        // upper-right (4) quadrants are on, which corresponds to the '▀' character.
        let chars: [&str; 16] = [
            " ", // 0000
            "▗", // 0001
            "▖", // 0010
            "▄", // 0011
            "▝", // 0100
            "▐", // 0101
            "▞", // 0110
            "▟", // 0111
            "▘", // 1000
            "▚", // 1001
            "▌", // 1010
            "▙", // 1011
            "▀", // 1100
            "▜", // 1101
            "▛", // 1110
            "█", // 1111
        ];


        // Iterate over character columns, building each one from quadrants.
        for x in 0..w_waveform {
            let left_sample_index = sample_offset + x * 2;
            let right_sample_index = left_sample_index + 1;

            if right_sample_index >= samples.len() {
                break;
            }

            let mut column_masks: std::collections::HashMap<i16, u8> = std::collections::HashMap::new();

            let sample_left = samples[left_sample_index];
            let sample_right = samples[right_sample_index];

            // --- Process Left Sample ---
            if sample_left != 0 {
                let mut height_f = calculate_height(sample_left);
                if height_f == height_f.floor() && height_f > 0.0 {
                    height_f -= 0.0001;
                }
                let height_int = height_f.floor() as i16;
                let height_frac = height_f.fract();

                if sample_left > 0 { // Positive wave, grows downward
                    for y in mid_upper..height_int {
                        *column_masks.entry(y).or_insert(0) |= 0b1010; // UL + LL bits
                    }
                    if height_frac >= 0.5 {
                        // FIX: A downward wave should fill the top of the cell, but visually it's reversed.
                        // Using Lower-Left quadrant to correct the output.
                        *column_masks.entry(height_int).or_insert(0) |= 0b0010; // LL bit
                    }
                } else { // Negative wave, grows upward
                    for y in (height_int + 1)..=mid_lower {
                        *column_masks.entry(y).or_insert(0) |= 0b1010; // UL + LL bits
                    }
                    if height_frac >= 0.5 {
                        // FIX: An upward wave should fill the bottom of the cell, but visually it's reversed.
                        // Using Upper-Left quadrant to correct the output.
                        *column_masks.entry(height_int).or_insert(0) |= 0b1000; // UL bit
                    }
                }
            }

            // --- Process Right Sample ---
            if sample_right != 0 {
                let mut height_f = calculate_height(sample_right);
                if height_f == height_f.floor() && height_f > 0.0 {
                    height_f -= 0.0001;
                }
                let height_int = height_f.floor() as i16;
                let height_frac = height_f.fract();

                if sample_right > 0 { // Positive
                    for y in mid_upper..height_int {
                        *column_masks.entry(y).or_insert(0) |= 0b0101; // UR + LR bits
                    }
                    if height_frac >= 0.5 {
                        // FIX: Using Lower-Right quadrant to correct the output.
                        *column_masks.entry(height_int).or_insert(0) |= 0b0001; // LR bit
                    }
                } else { // Negative
                    for y in (height_int + 1)..=mid_lower {
                        *column_masks.entry(y).or_insert(0) |= 0b0101; // UR + LR bits
                    }
                    if height_frac >= 0.5 {
                        // FIX: Using Upper-Right quadrant to correct the output.
                        *column_masks.entry(height_int).or_insert(0) |= 0b0100; // UR bit
                    }
                }
            }

            // --- Render the completed column to the main grid ---
            for (y, mask) in column_masks {
                if mask > 0 {
                    waveform_grid.insert((x as i16, y), chars[mask as usize]);
                }
            }
        }


        // eq calculation
        let fft_samples = &samples[sample_offset..sample_offset + fft_size];
        let bins = perform_fft(fft_samples);

        // loop through display bars
        for i in 0..w_eq {
            let start_bin = bar_bin_boundaries[i];
            let end_bin = bar_bin_boundaries[i + 1];

            // find peak magnitude in the bins covered by this bar
            let peak_magnitude = bins[start_bin..end_bin]
                .iter()
                .fold(0.0f32, |a, &b| a.max(b));

            // apply custom scaling
            let scaling_factor = 30_000_000.0;
            let scaled_magnitude = peak_magnitude / scaling_factor;

            // convert to decibels, add small epsilon to avoid log(0)
            let db = 20.0 * (scaled_magnitude + 1e-6).log10();
            // map db range [-50db, 0db] to a height percentage [0.0, 1.0]
            let min_db = -50.0;
            let max_db = 0.0;
            let height_percent = ((db - min_db) / (max_db - min_db)).clamp(0.0, 1.0);

            let new_height = height_percent * (h_eq - 1) as f32;

            // apply smoothing/gravity
            eq_bar_heights[i] = new_height.max(eq_bar_heights[i] * falloff_speed);

            // add to grid
            for y in 0..eq_bar_heights[i].floor() as i16 {
                eq_grid.insert((i as i16, y), "█");
            }
        }

        // waveform rendering
        let mut waveform_text = String::with_capacity((w_waveform + 1) * h_waveform);
        for y in (0..h_waveform).rev() { // draw bottom up
            for x in 0..w_waveform {
                waveform_text.push_str(waveform_grid.get(&(x as i16, y as i16)).unwrap_or(&" "));
            }
            waveform_text.push('\n');
        }

        // eq rendering
        let mut eq_text = String::with_capacity((w_eq + 1) * h_eq);
        for y in (0..h_eq).rev() {
            for x in 0..w_eq {
                eq_text.push_str(eq_grid.get(&(x as i16, y as i16)).unwrap_or(&" "));
            }
            eq_text.push('\n');
        }

        let mut text = format!("Max Amp: {max}  |  Time: {:.2}s", current_time as f32 / 1000.0);
        text += &(" ".repeat(w_waveform) + "\n");
        text += &(" ".repeat(w_waveform) + "\n");
        text += &waveform_text;
        text += &(" ".repeat(w_waveform) + "\n");
        text += &(" ".repeat(w_waveform) + "\n");
        text += &(" ".repeat(w_waveform) + "\n");
        text += &(" ".repeat(w_waveform) + "\n");
        text += &(" ".repeat(w_waveform) + "\n");
        text += &eq_text;

        print!("{text}");

    }

    toggle_alt_terminal(false);
    print!("\x1b[?25h"); // enable cursor

    Ok(())
}