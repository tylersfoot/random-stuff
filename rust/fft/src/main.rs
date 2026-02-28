use nannou::{draw::mesh::vertex::Color, geom::tri::{IterFromVertices, iter_from_vertices}, prelude::*};
use std::{
    fs::File, sync::Arc, thread, time::{Duration, Instant}, vec
};
use rustfft::{
    FftPlanner,
    num_complex::Complex,
};
use rodio::Decoder;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ringbuf::{HeapCons, HeapRb, SharedRb, storage::Heap, traits::*, wrap::caching::Caching};

// section of display in the window
#[derive(Debug, Clone, Copy)]
struct Section {
    // all dimensions in pixels
    // position is top left corner
    pos_x: f32,
    pos_y: f32,
    width: f32,
    height: f32,
}

fn main() {
    nannou::app(model)
        .event(event)
        .update(update)
        .view(view)
        .run();
}

struct Model {
    config: Config,
    realtime: bool, // are we capturing live audio or visualizing a pre-recorded file?
    _window: window::Id,

    // fps calculation
    last_frame_time: Instant,
    real_fps: f64,

    // realtime = true; use cpal
    audio_consumer: Option<HeapCons<f32>>,
    _cpal_audio_stream: Option<cpal::Stream>,
    sample_history: Vec<f32>, // a sliding window of recent samples
    buffer_size: usize, // how many samples to keep in the sliding window

    // realtime = false; use rodio + hound
    rodio_player: Option<rodio::Player>,
    _rodio_stream_handle: Option<rodio::MixerDeviceSink>,
    samples: Vec<f32>,
    max_sample: f32,
    sample_rate: u32,

    // waveform
    waveform_data: Vec<(f32, f32)>, // unscaled x,y points from 0-1

    // FFT/EQ
    eq_data: Vec<(f32, f32)>, // unscaled x,y points from 0-1
    fft: std::sync::Arc<dyn rustfft::Fft<f32>>,
    fft_window: Vec<f32>,
    fft_frequency_per_bin: f32,
    fft_bin_boundaries: Vec<usize>,
    fft_bin_display_heights: Vec<f32>,
    first_valid_i: usize,
}

struct Config {
    realtime_db_boost: f32,

    // waveform
    waveform_section: Section,
    waveform_points: usize, // polyline points
    waveform_thickness: f32, // polyline thickness
    waveform_zoom: f32,

    // FFT/EQ
    eq_section: Section,
    eq_thickness: f32, // polyline thickness
    eq_falloff_speed: f32,
    eq_freq_range: (f32, f32),
    fft_size: usize,
}

fn model(app: &App) -> Model {
    let realtime = true;

    let mut audio_consumer = None;
    let mut _cpal_audio_stream = None;
    let mut rodio_player = None;
    let mut _rodio_stream_handle = None;
    let mut sample_rate: u32 = 44100;
    let mut samples: Vec<f32> = Vec::new();
    let mut max_sample = 1.0;

    if realtime {
        // init ring buffer
        let rb = HeapRb::<f32>::new(44100 * 10); // 10 second buffer at 44.1kHz
        let (mut producer, consumer) = rb.split();
        // init cpal input stream
        let host = cpal::default_host();
        let device = host.default_output_device().expect("Failed to get default output device");
        let config = device.default_output_config().expect("Failed to get default output config");
        let channels = config.channels() as usize;
        // let sample_format = config.sample_format();
        sample_rate = config.sample_rate();

        // build the input stream to capture audio from the default input device
        // runs in a background thread
        let stream = device.build_input_stream(
            &config.clone().into(),
            move |data: &[f32], _: &_| {
                // average stereo data if present
                for frame in data.chunks(channels) {
                    let mut mono_sample = 0.0;
                    for &sample in frame {
                        mono_sample += sample;
                    }
                    mono_sample /= channels as f32;
                    let _ = producer.try_push(mono_sample);
                }
            },
            |err| eprintln!("Stream error: {err}"),
            None,
        ).unwrap();

        // print info
        println!("Device: {:?}", device.description());
        println!("Config: {:?}", config);

        // start the background audio thread
        stream.play().unwrap();

        audio_consumer = Some(consumer);
        _cpal_audio_stream = Some(stream);
    } else {
        let path = "sstv.wav"; // audio file path

        // -------- rodio playback --------
        let stream_handle = rodio::DeviceSinkBuilder::open_default_sink()
            .expect("open default audio stream");
        let player = rodio::Player::connect_new(stream_handle.mixer());
        let file = File::open(path).expect("Failed to open audio file");
        let source = Decoder::try_from(file).unwrap();
        player.pause();
        player.append(source);
        player.set_volume(0.01);

        // -------- hound wav reading --------
        let mut reader = hound::WavReader::open(path).unwrap();
        // audio specification (channels, sample rate, bits per sample, sample format)
        let spec = reader.spec();
        println!("Audio Spec: {spec:?}");
        sample_rate = reader.spec().sample_rate;
        // read the audio samples; type depends on bit depth (i16, i24, i32, f32)
        samples = match spec.sample_format {
            hound::SampleFormat::Int => reader.samples::<i32>().filter_map(Result::ok).map(|s| s as f32).collect(),
            hound::SampleFormat::Float => reader.samples::<f32>().filter_map(Result::ok).collect(),
        };

        println!("Successfully read {} samples.", samples.len());
        // if stereo, only take one channel for visualizing
        if reader.spec().channels > 1 {
            let stereo_samples: Vec<f32> = samples.clone();
            samples = stereo_samples.iter().step_by(2).cloned().collect();
        }
        // find max sample value for normalization;
        max_sample = get_max(&samples);

        rodio_player = Some(player);
        _rodio_stream_handle = Some(stream_handle);
    }

    let window = app.new_window()
        .size(1100, 700)
        .view(view)
        .build()
        .unwrap();
    app.set_loop_mode(LoopMode::RefreshSync);

    wait(1000);

    // -------- config and setup --------
    let waveform_section = Section {
        pos_x: 0.0,
        pos_y: 50.0,
        width: 1100.0,
        height: 300.0,
    };
    let eq_section = Section {
        pos_x: 0.0,
        pos_y: 400.0,
        width: 1100.0,
        height: 300.0,
    };

    // EQ setup
    let fft_size = 2048; // how many samples to analyze for the equalizer
    let buffer_size = fft_size * 2;
    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(fft_size);
    // frequency increase per fft bin
    let fft_frequency_per_bin = sample_rate as f32 / fft_size as f32;
    let eq_freq_range = (20.0, 20000.0);

    // pre-calculate which FFT bins correspond to each bar/column/pixel in the EQ display
    let mut fft_bin_boundaries = vec![0; eq_section.width as usize + 1];
    for i in 0..=eq_section.width as usize {
        let freq_boundary = eq_freq_range.0 * (eq_freq_range.1 / eq_freq_range.0).powf(i as f32 / eq_section.width);
        let bin_index = (freq_boundary / fft_frequency_per_bin).floor() as usize;
        fft_bin_boundaries[i] = bin_index.min(fft_size / 2); // cap at max bin index
    }

    // find the first pixel column that actually contains a bin
    let mut first_valid_i = 0;
    for i in 0..eq_section.width as usize {
        if fft_bin_boundaries[i] < fft_bin_boundaries[i + 1] {
            first_valid_i = i;
            break;
        }
    }

    // create a Hann Window to reduce spectral leakage in the FFT
    let fft_window= (0..fft_size)
    .map(|i| 0.5 * (1.0 - (2.0 * std::f32::consts::PI * i as f32 / (fft_size - 1) as f32).cos()))
    .collect();
    
    // bin heights for display, smoothed with eq_falloff_speed
    let fft_bin_display_heights= vec![0.0; eq_section.width as usize];

    if !realtime {
        rodio_player.as_ref().unwrap().play();
    }

    let config = Config {
        realtime_db_boost: 20.0,
        // waveform
        waveform_section,
        waveform_points: waveform_section.width as usize / 2, // 1 point per pixel
        // waveform_points: 100,
        waveform_thickness: 2.0,
        waveform_zoom: 1.0,
        // FFT/EQ
        eq_section,
        eq_thickness: 2.0,
        eq_falloff_speed: 0.6, // % falloff per second
        eq_freq_range,
        fft_size,
    };

    Model {
        config,
        realtime,
        _window: window,
        last_frame_time: Instant::now(),
        real_fps: 60.0,
        audio_consumer,
        _cpal_audio_stream,
        sample_history: vec![0.0; buffer_size],
        buffer_size,
        rodio_player,
        _rodio_stream_handle,
        sample_rate,
        samples,
        max_sample,
        // waveform data
        waveform_data: Vec::new(),
        // FFT/EQ data
        eq_data: Vec::new(),
        fft,
        fft_window,
        fft_frequency_per_bin,
        fft_bin_boundaries,
        fft_bin_display_heights,
        first_valid_i,
    }
}

fn update(_app: &App, model: &mut Model, _update: Update) {
    // fps calculation
    let now = Instant::now();
    let delta = now.duration_since(model.last_frame_time).as_secs_f64();
    model.last_frame_time = now;
    // simple moving average to smooth out FPS display
    if delta > 0.0 {
        let current_fps = 1.0 / delta;
        model.real_fps = (model.real_fps * 0.9) + (current_fps * 0.1);
    }
    let realtime = model.realtime;

    if !realtime {
        let current_time = model.rodio_player.as_ref().unwrap().get_pos().as_millis();

        let sample_offset = (current_time as usize * model.sample_rate as usize) / 1000;
        // dont read past the end of the samples
        if sample_offset + model.config.fft_size >= model.samples.len() {
            exit();
        }
    } else {
        let new_samples: Vec<f32> = model.audio_consumer
            .as_mut()
            .unwrap()
            .pop_iter()
            .collect();
        
        // shift old samples out and push new ones in
        let new_len = new_samples.len();
        if new_len > 0 {
            if new_len >= model.buffer_size {
                // got more samples than we can hold, take the newest ones
                model.sample_history.copy_from_slice(&new_samples[new_len - model.buffer_size..]);
            } else {
                // shift old samples left, add new samples at the end
                model.sample_history.copy_within(new_len.., 0);
                model.sample_history[model.buffer_size - new_len..].copy_from_slice(&new_samples);
            }
        }
    }

    let samples = if realtime {
        // boost gain for realtime audio for higher visibility
        let gain = 10.0f32.powf(model.config.realtime_db_boost / 20.0);
        &model.sample_history
            .iter()
            .map(|s| {
                (s * gain).clamp(-model.max_sample, model.max_sample)
            })
            .collect::<Vec<f32>>()
    } else {
        &model.samples
    };

    let sample_offset = if realtime {
        0 // always start at the end of the history buffer for live audio
    } else {
        (model.rodio_player.as_ref().unwrap().get_pos().as_millis() as usize * model.sample_rate as usize) / 1000
    };
    
    { // -------- waveform calculation --------
        // display less to account for zero crossing offset
        let sample_range = (model.buffer_size as f32 * 0.5) as usize;
        // buffer to add smoothed/lowpassed samples
        let mut bass_buffer: Vec<f32> = vec![0.0; model.buffer_size];
        let noise_mult = 0.2; // ignore small fluctuations around zero
        let threshold = 0.0; // the value that represents the "zero" line for crossing detection, typically 0 for signed audio
        let mut crossed = false; // have we dipped below the noise floor
        let smoothing = 0.005; // how much to smooth/lowpass the waveform - 0.0 (max) to 1.0 (none)
        let mut crossing_offset = 0; // how many samples to shift the window for zero crossing

        // apply smoothing/lowpass to the samples to reduce noise and make triggering more stable
        if smoothing < 1.0 {
            // to prevent a spike at the beginning, we start with a value that is pre-smoothed from a few samples before the buffer
            let mut current_value = samples[0];
            if !realtime {
                let pre_samples = (8.0 / smoothing) as usize;
                let pre_smooth_offset = sample_offset.saturating_sub(pre_samples);
                current_value = samples[pre_smooth_offset];

                for sample in &samples[pre_smooth_offset..=sample_offset] {
                    let target_value = *sample;
                    current_value += (target_value - current_value) * (smoothing);
                }
            } else {
                // for realtime, smooth first couple samples instead
                for sample in &samples[0..8] {
                    let target_value = *sample;
                    current_value += (target_value - current_value) * (smoothing);
                }
            }

            // start filling the bass buffer with smoothed values
            bass_buffer[0] = current_value;
            for i in 1..bass_buffer.len() {
                let target_value = samples[sample_offset + i];
                current_value += (target_value - current_value) * (smoothing);
                bass_buffer[i] = current_value;
            }
        } else {
            // just copy raw samples without smoothing
            let end_offset = bass_buffer.len() + sample_offset;
            bass_buffer.copy_from_slice(&samples[sample_offset..end_offset]);
        }
        
        // look ahead in the samples to find a zero crossing for stable triggering
        let max_sample = get_max(&bass_buffer); // find max in the bass buffer for normalized thresholding
        let noise_gap = max_sample * noise_mult; // calculate noise gap based on max amplitude
        // restricts max offset to sample_range to allow full display
        for i in 0..(bass_buffer.len() - sample_range) - 1 {
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

        let points = model.config.waveform_points; // resolution/number of points
        let base_offset = sample_offset + crossing_offset;
        model.waveform_data = Vec::with_capacity(points);

        // loop over each point and calculate the waveform height
        for i in 0..points {
            // normalized position/percent from 0 to 1
            let start_index = base_offset + (i * sample_range) / points;
            let end_index = base_offset + ((i + 1) * sample_range) / points;

            if end_index > samples.len() {
                break;
            }

            let slice = &samples[start_index..end_index];
            if slice.is_empty() {
                continue;
            }

            let mut peak_sample = 0.0;
            let mut max_abs = -1.0;

            for &sample in slice {
                let abs_val = sample.abs();
                if abs_val > max_abs {
                    max_abs = abs_val;
                    peak_sample = sample;
                }
            }

            // debug: no averaging
            // let peak_sample = samples[start_index];

            let x = i as f32 / points as f32;
            let y = peak_sample / model.max_sample + 1.0 / 2.0;
            model.waveform_data.push((x.clamp(0.0, 1.0), y.clamp(0.0, 1.0)));
        }
    }

    { // -------- EQ calculation --------
        let fft_samples = if realtime {
            let start_fft = samples.len().saturating_sub(model.config.fft_size);
            &samples[start_fft..]
        } else {
            &samples[sample_offset..sample_offset + model.config.fft_size]
        };

        let bins = perform_fft(fft_samples, &model.fft, &model.fft_window);
        model.eq_data = Vec::with_capacity(model.config.eq_section.width as usize);

        let first_i = model.first_valid_i; // first pixel column that has a corresponding FFT bin
        let last_i = model.config.eq_section.width as usize - 1; // last pixel column

        // loop through display bars
        for i in 0..model.config.eq_section.width as usize {
            let start_bin = model.fft_bin_boundaries[i];
            let end_bin = model.fft_bin_boundaries[i + 1];

            // skip rendering if no bins correspond to this bar
            if start_bin >= end_bin {
                continue;
            }

            // find peak magnitude in the bins covered by this bar
            let peak_magnitude = bins[start_bin..end_bin]
                .iter()
                .fold(0.0f32, |a, &b| a.max(b));

            // apply custom scaling
            let scaling_factor = if realtime {
                model.config.fft_size as f32 / 2.0
                // 1.0
            } else {
                30_000_000.0
            };
            let scaled_magnitude = peak_magnitude / scaling_factor;

            // convert to decibels, add small epsilon to avoid log(0)
            let db = 20.0 * (scaled_magnitude + 1e-6).log10();
            // map db range [-50db, 0db] to a height percentage [0.0, 1.0]
            let min_db = -60.0;
            let max_db = 0.0;
            let height = ((db - min_db) / (max_db - min_db)).clamp(0.0, 1.0);

            // apply smoothing/gravity
            let falloff = model.config.eq_falloff_speed.powf(delta as f32 * 100.0);
            let height = height.max(model.fft_bin_display_heights[i] * falloff);
            model.fft_bin_display_heights[i] = height;

            // calculate x based on shifted range
            let x = if last_i > first_i {
                (i - first_i) as f32 / (last_i - first_i) as f32
            } else {
                0.0
            };
            let y = height;
            model.eq_data.push((x.clamp(0.0, 1.0), y.clamp(0.0, 1.0)));
        }
    }

    // force CPU to sleep to cap fps
    let target_frame_time = Duration::from_micros(10_000); // ~100fps cap
    let elapsed = now.elapsed();
    if elapsed < target_frame_time {
        thread::sleep(target_frame_time - elapsed);
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    draw.background().color(BLACK);
    let win = app.window_rect();

    // helper function to convert from normalized (0-1) coordinates in a section to screen coordinates
    let section_to_screen = |section: &Section, x: f32, y: f32| -> Point2 {
        let screen_x = section.pos_x + x * section.width;
        let screen_y = section.pos_y + (1.0 - y) * section.height;
        // convert from top-left origin to nannou's center origin
        let screen_x = screen_x - win.w() / 2.0;
        let screen_y = win.h() / 2.0 - screen_y;
        pt2(screen_x, screen_y)
    };

    // draw waveform
    let waveform_vertices: Vec<Point2> = model.waveform_data.iter().map(|&(x, y)| {
        section_to_screen(&model.config.waveform_section, x, y)
    }).collect();

    let mut waveform_mesh_vertices = Vec::new();
    let top_color = srgba(110.0 / 255.0, 192.0 / 255.0, 126.0 / 255.0, 0.8);
    let bottom_color = srgba(60.0 / 255.0, 98.0 / 255.0, 68.0 / 255.0, 0.8);
    let baseline = section_to_screen(&model.config.waveform_section, 0.0, 0.5).y;
    for window in waveform_vertices.windows(2) {
        let p0 = window[0]; // left point
        let p1 = window[1]; // right point

        let top_left = (Point3::new(p0.x, p0.y, 0.0), top_color);
        let bottom_left = (Point3::new(p0.x, baseline, 0.0), bottom_color);
        let top_right = (Point3::new(p1.x, p1.y, 0.0), top_color);
        let bottom_right = (Point3::new(p1.x, baseline, 0.0), bottom_color);

        // triangle 1: left side of the quad
        waveform_mesh_vertices.push(top_left);
        waveform_mesh_vertices.push(bottom_left);
        waveform_mesh_vertices.push(top_right);

        // triangle 2: right side of the quad
        waveform_mesh_vertices.push(top_right);
        waveform_mesh_vertices.push(bottom_left);
        waveform_mesh_vertices.push(bottom_right);
    }

    draw.mesh().tris_colored(iter_from_vertices(waveform_mesh_vertices));

    draw.polyline()
        .weight(model.config.waveform_thickness)
        .join_round()
        .points(waveform_vertices)
        .rgb8(140, 255, 160);

    // draw EQ
    let eq_vertices: Vec<Point2> = model.eq_data.iter().map(|&(x, y)| {
        section_to_screen(&model.config.eq_section, x, y)
    }).collect();

    let mut eq_mesh_vertices = Vec::new();
    let top_color = srgba(122.0 / 255.0, 15.0 / 255.0, 80.0 / 255.0, 1.0);
    let bottom_color = srgba(77.0 / 255.0, 12.0 / 255.0, 88.0 / 255.0, 1.0);
    let baseline = section_to_screen(&model.config.eq_section, 0.0, 0.0).y;
    for window in eq_vertices.windows(2) {
        let p0 = window[0]; // left point
        let p1 = window[1]; // right point

        let top_left = (Point3::new(p0.x, p0.y, 0.0), top_color);
        let bottom_left = (Point3::new(p0.x, baseline, 0.0), bottom_color);
        let top_right = (Point3::new(p1.x, p1.y, 0.0), top_color);
        let bottom_right = (Point3::new(p1.x, baseline, 0.0), bottom_color);

        // triangle 1: left side of the quad
        eq_mesh_vertices.push(top_left);
        eq_mesh_vertices.push(bottom_left);
        eq_mesh_vertices.push(top_right);

        // triangle 2: right side of the quad
        eq_mesh_vertices.push(top_right);
        eq_mesh_vertices.push(bottom_left);
        eq_mesh_vertices.push(bottom_right);
    }

    draw.mesh().tris_colored(iter_from_vertices(eq_mesh_vertices));

    draw.polyline()
        .weight(model.config.eq_thickness)
        .join_round()
        .points(eq_vertices)
        .rgb8(255, 90, 120);

    // fps counter
    let fps_display = model.real_fps;
    let fps_text = format!("{fps_display:.0}");
    draw.text(&fps_text)
        .font_size(24)
        .color(WHITE)
        .xy(win.top_left() + vec2(20.0, -15.0));

    draw.to_frame(app, &frame).unwrap();
}

fn event(_app: &App, _model: &mut Model, _event: Event) {
}

fn perform_fft(audio_samples: &[f32], fft: &std::sync::Arc<dyn rustfft::Fft<f32>>, window: &[f32]) -> Vec<f32> {
    let fft_size = audio_samples.len();

    // apply window function and convert to complex format for FFT input
    let mut buffer: Vec<Complex<f32>> = audio_samples
    .iter()
    .zip(window) // zip with the window
    .map(|(&sample, &win_val)| {
        // apply window to sample
        Complex::new(sample * win_val, 0.0)
    })
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

fn get_max(samples: &[f32]) -> f32 {
    samples.iter().map(|s| s.abs()).fold(f32::NEG_INFINITY, |a, b| a.max(b))
}

fn wait(millis: u64) {
    thread::sleep(Duration::from_millis(millis));
}

fn exit() {
    println!("Exiting...");
    std::process::exit(0);
}