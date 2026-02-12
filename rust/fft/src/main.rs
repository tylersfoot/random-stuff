use std::{
    collections::HashMap,
    fs::File,
    io::{
        BufWriter,
        Write,
        stdout
    },
    thread,
    time::Duration,
    vec
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

fn perform_fft(audio_samples: &[i32], fft: &std::sync::Arc<dyn rustfft::Fft<f32>>, window: &[f32]) -> Vec<f32> {
    let fft_size = audio_samples.len();

    // apply window function and convert to complex format for FFT input
    let mut buffer: Vec<Complex<f32>> = audio_samples
    .iter()
    .zip(window) // zip with the window
    .map(|(&sample, &win_val)| {
        // apply window to sample
        Complex::new(sample as f32 * win_val, 0.0)
    })
    .collect();

    // let mut buffer: Vec<Complex<f32>> = audio_samples
    //     .iter()
    //     .map(|&sample| Complex::new(sample as f32, 0.0))
    //     .collect();

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
    let path = "orbital.wav"; // audio file path

    // -------- rodio playback
    let stream_handle = rodio::OutputStreamBuilder::open_default_stream()
        .expect("open default audio stream");
    let sink = rodio::Sink::connect_new(stream_handle.mixer());
    let file = File::open(path).expect("Failed to open audio file");
    let source = Decoder::try_from(file).unwrap();
    sink.pause();
    sink.append(source);
    sink.set_volume(0.2);

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

    let show_waveform = true;
    let show_eq = true;
    let waveform_zoom = 3; // how many samples per character column, higher = more zoomed out
    let w_waveform = 350;
    let w_eq = 350;
    let h_waveform = 40;
    let h_eq = 30;
    let mut waveform_grid = HashMap::new();
    let mut eq_grid = HashMap::new();
    //                              add extra *2 overflow protection w/test display
    let mut bass_buffer = vec![0; w_waveform * (waveform_zoom * 2)];

    // find the max amplitude for normalization
    let get_max = |samples: &[i32]| -> i32 {
        samples.iter().map(|s| s.abs()).max().unwrap_or(i32::MAX)
    };
    let max = get_max(&samples);
    // let middle = max / 2; // the 'zero' line in the waveform

    toggle_alt_terminal(true);
    print!("\x1b[?25l"); // disable cursor

    // let start_time = Instant::now();
    let mut current_time;
    let mut previous_time = 0;
    let end_time = 180 * 1000; // what time to end (in sec)
    let target_fps = 165; // fps to render at
    let render_delay = 1_000 / target_fps; // pass this time to render next frame
    let mut time_buffer = 0; // time elapsed, for delaying rendering

    // EQ setup
    let fft_size = 2048; // how many samples to analyze for the equalizer
    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(fft_size);
    // frequency increase per fft bin
    let frequency_per_bin = sample_rate as f32 / fft_size as f32;
    let mut bar_bin_boundaries = vec![0; w_eq + 1];
    let min_freq: f32 = 20.0; // min frequency to display
    let max_freq: f32 = 20000.0; // max frequency
    let log_scale = (max_freq / min_freq).ln();

    // create a window function to reduce spectral leakage in the FFT
    let window: Vec<f32> = (0..fft_size)
    .map(|i| 0.5 * (1.0 - (2.0 * std::f32::consts::PI * i as f32 / (fft_size - 1) as f32).cos()))
    .collect();

    // horizontal log scaling
    for i in 0..=w_eq {
        let freq_boundary = min_freq * (max_freq / min_freq).powf(i as f32 / w_eq as f32);
        let bin_index = (freq_boundary / frequency_per_bin).floor() as usize;
        bar_bin_boundaries[i] = bin_index.min(fft_size / 2); // cap at max bin index
    }
    
    // EQ smoothing
    let mut eq_bar_heights: Vec<f32> = vec![0.0; w_eq];
    let falloff_speed = 0.95; // 5% falloff per frame

    let delay = 000; // ms visual offset, increase if visuals are early

    // helper to convert a sample value to a height on the waveform display
    let calculate_height = |sample: i32| -> f32 {
        let mut height = sample as f32 / max as f32; // -1.0 to 1.0
        height = height / 2.0 + 0.5; // 0.0 to 1.0
        height * h_waveform as f32 // 0.0 to h-1
    };

    // pre-calculate the middle lines for filling logic in the waveform rendering
    let mid_upper = (h_waveform / 2) as i16;
    let mid_lower = (h_waveform / 2 - 1) as i16;

    // characters for rendering the waveform based on which quadrants of the cell are filled
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

    // helper to get interpolated magnitude for a non-integer bin index
    let get_interpolated_magnitude = |bins: &[f32], index: f32| -> f32 {
        let len = bins.len() as f32;
        if index < 0.0 || index >= len - 1.0 {
            return 0.0;
        }
        
        let floor_idx = index.floor() as usize;
        let ceil_idx = index.ceil() as usize;
        let t = index - floor_idx as f32; // fractional part (0.0 to 1.0)
        
        let val_low = bins[floor_idx];
        let val_high = bins[ceil_idx];
        
        // linear interpolation: (1 - t) * low + t * high
        val_low * (1.0 - t) + val_high * t
    };

    // use a buffered writer for more efficient output to the terminal
    let stdout = stdout();
    let mut handle = BufWriter::new(stdout.lock());
    let spacer_line = " ".repeat(w_waveform);

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

        // clear grid
        if show_waveform {
            for y in 0..h_waveform {
                for x in 0..w_waveform {
                    waveform_grid.insert((x as i16, y as i16), " ");
                }
            }
        }
        if show_eq {
            for y in 0..h_eq {
                for x in 0..w_eq {
                    eq_grid.insert((x as i16, y as i16), " ");
                }
            }
        }

        // waveform calculation
        if show_waveform {
            bass_buffer.fill(0); // buffer to add smoothed/lowpassed samples
            let noise_mult = 0.2; // ignore small fluctuations around zero
            let threshold = 0; // the value that represents the "zero" line for crossing detection, typically 0 for signed audio
            let mut crossed = false; // have we dipped below the noise floor
            let smoothing = 0.005; // how much to smooth/lowpass the waveform - 0.0 (max) to 1.0 (none)
            let mut crossing_offset = 0; // how many samples to shift the window for zero crossing

            // apply smoothing/lowpass to the samples to reduce noise and make triggering more stable
            if smoothing < 1.0 {
                // to prevent a spike at the beginning, we start with a value that is pre-smoothed from a few samples before the buffer
                let pre_samples = (8.0 / smoothing) as usize;
                let pre_smooth_offset = sample_offset.saturating_sub(pre_samples);
                let mut current_value = samples[pre_smooth_offset] as f32;

                for sample in &samples[pre_smooth_offset..=sample_offset] {
                    let target_value = *sample as f32;
                    current_value += (target_value - current_value) * (smoothing);
                }

                // start filling the bass buffer with smoothed values
                bass_buffer[0] = current_value as i32;
                for i in 1..bass_buffer.len() {
                    let target_value = samples[sample_offset + i] as f32;
                    current_value += (target_value - current_value) * (smoothing);
                    bass_buffer[i] = current_value as i32;
                }
            } else {
                // just copy raw samples without smoothing
                let end_offset = bass_buffer.len() + sample_offset;
                bass_buffer.copy_from_slice(&samples[sample_offset..end_offset]);
            }
            
            // look ahead in the samples to find a zero crossing for stable triggering
            let max_sample = get_max(&bass_buffer); // find max in the bass buffer for normalized thresholding
            let noise_gap = (max_sample as f32 * noise_mult) as i32; // calculate noise gap based on max amplitude
            for i in 0..(bass_buffer.len() / 2) - 1 {
                let current_sample = bass_buffer[i];
                let next_sample = bass_buffer[i + 1];

                // first, look for a dip below the noise threshold
                if !crossed && current_sample < (threshold - noise_gap) {
                    crossed = true;
                }

                // then, look for the rising crossing back above the threshold + noise gap
                if crossed && current_sample <= (threshold + noise_gap) && next_sample > (threshold + noise_gap) {
                    crossing_offset = i; // shift the window by this many samples to align with the crossing
                    break;
                }
            }

            // iterate over character columns, building each one from quadrants
            for x in 0..w_waveform {
                let left_sample_index = sample_offset + crossing_offset + x * (2 * waveform_zoom);
                let right_sample_index = left_sample_index + waveform_zoom;

                if right_sample_index >= samples.len() {
                    break;
                }

                let mut column_masks: std::collections::HashMap<i16, u8> = std::collections::HashMap::new();

                let sample_left = samples[left_sample_index];
                let sample_right = samples[right_sample_index];
                // let sample_left = bass_buffer[left_sample_index - sample_offset];
                // let sample_right = bass_buffer[right_sample_index - sample_offset];

                // left sample
                if sample_left != 0 {
                    let mut height_f = calculate_height(sample_left);
                    if height_f == height_f.floor() && height_f > 0.0 {
                        height_f -= 0.0001;
                    }
                    let height_int = height_f.floor() as i16; // floored int height
                    let height_frac = height_f.fract(); // the fractional part, used to determine if we fill the half block

                    if sample_left > 0 { // positive: fills down
                        // fill down to the middle
                        for y in mid_upper..height_int {
                            *column_masks.entry(y).or_insert(0) |= 0b1010; // UL + LL
                        }

                        // if the height is above the middle of the cell, fill the lower half
                        if height_frac >= 0.5 {
                            *column_masks.entry(height_int).or_insert(0) |= 0b0010; // LL
                        }
                    } else { // negative: fills up
                        // fill up to the middle
                        for y in (height_int + 1)..=mid_lower {
                            *column_masks.entry(y).or_insert(0) |= 0b1010; // UL + LL
                        }
                        // if the height is below the middle of the cell, fill the upper half
                        if height_frac < 0.5 {
                            *column_masks.entry(height_int).or_insert(0) |= 0b1000; // UL
                        }
                    }
                }

                // right sample
                if sample_right != 0 {
                    let mut height_f = calculate_height(sample_right);
                    if height_f == height_f.floor() && height_f > 0.0 {
                        height_f -= 0.0001;
                    }
                    let height_int = height_f.floor() as i16;
                    let height_frac = height_f.fract();

                    if sample_right > 0 { // positive: fills down
                        // fill down to the middle
                        for y in mid_upper..height_int {
                            *column_masks.entry(y).or_insert(0) |= 0b0101; // UR + LR
                        }
                        // if the height is above the middle of the cell, fill the lower half
                        if height_frac >= 0.5 {
                            *column_masks.entry(height_int).or_insert(0) |= 0b0001; // LR
                        }
                    } else { // negative: fills up
                        // fill up to the middle
                        for y in (height_int + 1)..=mid_lower {
                            *column_masks.entry(y).or_insert(0) |= 0b0101; // UR + LR
                        }
                        // if the height is below the middle of the cell, fill the upper half
                        if height_frac < 0.5 {
                            *column_masks.entry(height_int).or_insert(0) |= 0b0100; // UR
                        }
                    }
                }

                // render the column based on the quadrant masks
                for (y, mask) in column_masks {
                    if mask > 0 {
                        waveform_grid.insert((x as i16, y), chars[mask as usize]);
                    }
                }
            }
        }

        // eq calculation
        if show_eq {
            let fft_samples = &samples[sample_offset..sample_offset + fft_size];
            let bins = perform_fft(fft_samples, &fft, &window);

            // loop through display bars
            for i in 0..w_eq {
                // calculate the frequency range that this bar covers using logarithmic scaling
                let f_start = min_freq * (log_scale * (i as f32 / w_eq as f32)).exp();
                let f_end = min_freq * (log_scale * ((i + 1) as f32 / w_eq as f32)).exp();
                let center_freq = (f_start + f_end) / 2.0;

                // calculate which FFT bins correspond to this frequency range
                let bin_resolution = sample_rate as f32 / fft_size as f32;
                let center_bin_idx = center_freq / bin_resolution;
                
                let bin_width = (f_end - f_start) / bin_resolution;

                let magnitude = if bin_width < 1.0 {
                    // if the bar covers less than 1 bin, use interpolation to estimate the magnitude at the center frequency
                    get_interpolated_magnitude(&bins, center_bin_idx)
                } else {
                    // otherwise, find the peak magnitude among the bins that fall within this bar's frequency range
                    let start_idx = (f_start / bin_resolution).floor() as usize;
                    let end_idx = (f_end / bin_resolution).ceil() as usize;
                    
                    // clamp indices to valid range and ensure start is less than end
                    let start = start_idx.clamp(0, bins.len());
                    let end = end_idx.clamp(0, bins.len());
                    
                    if start >= end {
                        get_interpolated_magnitude(&bins, center_bin_idx)
                    } else {
                        bins[start..end].iter().fold(0.0f32, |a, &b| a.max(b))
                    }
                };

                // let start_bin = bar_bin_boundaries[i];
                // let end_bin = bar_bin_boundaries[i + 1].max(start_bin + 1);

                // // find peak magnitude in the bins covered by this bar
                // let peak_magnitude = bins[start_bin..end_bin]
                //     .iter()
                //     .fold(0.0f32, |a, &b| a.max(b));

                // apply custom scaling
                let scaling_factor = 30_000_000.0;
                let scaled_magnitude = magnitude / scaling_factor;

                // convert to decibels, add small epsilon to avoid log(0)
                let db = 20.0 * (scaled_magnitude + 1e-6).log10();
                // map db range [-50db, 0db] to a height percentage [0.0, 1.0]
                let min_db = -50.0;
                let max_db = 0.0;
                let height_percent = ((db - min_db) / (max_db - min_db)).clamp(0.0, 1.0);

                let new_height = height_percent * (h_eq - 1) as f32;

                // apply smoothing/gravity
                eq_bar_heights[i] = new_height.max(eq_bar_heights[i] * falloff_speed);
            }

            // optional: apply additional smoothing across neighboring bars to reduce sharp spikes
            let mut smoothed_heights = eq_bar_heights.clone();

            for i in 1..w_eq - 1 {
                // simple average of the current bar and its immediate neighbors
                let prev = eq_bar_heights[i - 1];
                let curr = eq_bar_heights[i];
                let next = eq_bar_heights[i + 1];
                smoothed_heights[i] = (prev + curr + next) / 3.0;
            }

            eq_bar_heights = smoothed_heights;

            for (i, &height) in eq_bar_heights.iter().enumerate() {
                // add to grid
                for y in 0..height.floor() as i16 {
                    eq_grid.insert((i as i16, y), "█");
                }
            }
        }

        // waveform rendering
        let mut waveform_text = String::new();
        if show_waveform {
            waveform_text = String::with_capacity((w_waveform + 1) * h_waveform);
            for y in (0..h_waveform).rev() { // draw bottom up
                for x in 0..w_waveform {
                    waveform_text.push_str(waveform_grid.get(&(x as i16, y as i16)).unwrap_or(&" "));
                }
                waveform_text.push('\n');
            }
        }

        // eq rendering
        let mut eq_text = String::new();
        if show_eq {
            eq_text = String::with_capacity((w_eq + 1) * h_eq);
            for y in (0..h_eq).rev() {
                for x in 0..w_eq {
                    eq_text.push_str(eq_grid.get(&(x as i16, y as i16)).unwrap_or(&" "));
                }
                eq_text.push('\n');
            }
        }

        write!(handle, "\x1b[H").unwrap(); // cursor position to top left
        writeln!(handle, "Max Amp: {}  |  Time: {:.2}s", max, current_time as f32 / 1000.0).unwrap();

        writeln!(handle, "{spacer_line}").unwrap();
        writeln!(handle, "{spacer_line}").unwrap();

        if show_waveform {
            write!(handle, "{waveform_text}").unwrap();
        }

        writeln!(handle, "{spacer_line}").unwrap();
        writeln!(handle, "{spacer_line}").unwrap();
        writeln!(handle, "{spacer_line}").unwrap();
        writeln!(handle, "{spacer_line}").unwrap();
        writeln!(handle, "{spacer_line}").unwrap();

        if show_eq {
            write!(handle, "{eq_text}").unwrap();
        }

        handle.flush().unwrap();
    }

    toggle_alt_terminal(false);
    print!("\x1b[?25h"); // enable cursor

    Ok(())
}