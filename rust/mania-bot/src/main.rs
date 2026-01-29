use std::{process::exit, time::{Duration, Instant}};
use xcap::Monitor;
use rustautogui::RustAutoGui;
use rdev::{listen, EventType, Key};
use std::thread;
use std::io::{self, Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Script started!");

    // listener for W and Q keys (in separate thread)
    println!("Press 'W' to start, 'Q' to quit...");
    let (start_tx, start_rx) = std::sync::mpsc::channel();
    let (quit_tx, quit_rx) = std::sync::mpsc::channel();
    thread::spawn(move || {
        if let Err(error) = listen(move |event| {
            match event.event_type {
                EventType::KeyPress(Key::KeyW) => {
                    let _ = start_tx.send(());
                }
                EventType::KeyPress(Key::KeyQ) => {
                    let _ = quit_tx.send(());
                }
                _ => {}
            }
        }) {
            println!("Error: {error:?}")
        }
    });

    let rag = RustAutoGui::new(false)?;

    let monitors = Monitor::all()?;
    let monitor = monitors
        .into_iter()
        .find(|m| m.is_primary().unwrap_or(false))
        .expect("No primary monitor found");

    const KEYS: usize = 7;
    let sample_y = 1190; // y position to sample
    let sample_x_start = 1475; // starting x position to sample
    let sample_spacing = 90; // spacing between samples
    // 1130, 900
    // 1230, 900
    // 1330, 900
    // 1430, 900

    let mut state_keys = [false; KEYS]; // true = keydown, false = keyup
    let mut state_colors = [false; KEYS]; // true = white (note)

    let _ = start_rx.recv();
    println!("Started!");

    let mut sample_count = 0u64;
    let mut key_press_count = 0u64;
    let mut last_print = Instant::now();

    // main loop
    loop {
        // check for quit signal
        if quit_rx.try_recv().is_ok() {
            println!("Exiting...");
            exit(0);
        }
        // sample screen at positions
        let image = monitor.capture_region(
            sample_x_start,
            sample_y,
            sample_spacing * KEYS as u32,
            5,
        )?;

        // check determined point's color for white
        for (key, color) in state_colors.iter_mut().enumerate() {
            *color = false;
            for h in 0..5 {
                let pixel = image.get_pixel(sample_spacing * key as u32, h);
                if (pixel[0] >= 240) && (pixel[1] >= 240) && (pixel[2] >= 240) {
                    // white
                    *color = true;
                }
            }
        }

        // handle key presses
        for (usize, is_white) in state_colors.iter().enumerate() {
            if *is_white && !state_keys[usize] {
                // key down
                match usize {
                    0 => rag.key_down("s")?,
                    1 => rag.key_down("d")?,
                    2 => rag.key_down("f")?,
                    3 => rag.key_down("g")?,
                    4 => rag.key_down("j")?,
                    5 => rag.key_down("k")?,
                    6 => rag.key_down("l")?,
                    _ => {}
                }
                state_keys[usize] = true;
                key_press_count += 1;
            } else if !*is_white && state_keys[usize] {
                // key up
                match usize {
                    0 => rag.key_up("s")?,
                    1 => rag.key_up("d")?,
                    2 => rag.key_up("f")?,
                    3 => rag.key_up("g")?,
                    4 => rag.key_up("j")?,
                    5 => rag.key_up("k")?,
                    6 => rag.key_up("l")?,
                    _ => {}
                }
                state_keys[usize] = false;
            }
        }

        sample_count += 1;
        let elapsed = last_print.elapsed();
        if elapsed >= Duration::from_millis(1000) {
            let samples_per_sec = sample_count / elapsed.as_secs().max(1);
            let keys_per_sec = key_press_count / elapsed.as_secs().max(1);
            print!("\rSamples/sec: {samples_per_sec} | Keys/sec: {keys_per_sec}     ");
            io::stdout().flush()?;
            sample_count = 0;
            key_press_count = 0;
            last_print = Instant::now();
        }
    }

    Ok(())
}
