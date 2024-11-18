use std::collections::VecDeque;
use std::io;
use std::time::{Duration, Instant};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget},
    DefaultTerminal, Frame,
};
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg32;
use ratatui::style::Color;
use noise::{NoiseFn, Perlin, Seedable};

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    // let app_result = App::default().run(&mut terminal);
    let app_result = App {
        seed: 0.0,
        start_time: Instant::now(),
        time_of_day: 0.0,
        last_frame_time: Instant::now(),
        fps: 0.0,
        frame_times: VecDeque::new(),
        counter: 0,
        exit: false,
    }
        .run(&mut terminal);
    ratatui::restore();
    app_result
}

#[derive(Debug)]
pub struct App {
    seed: f64,
    start_time: Instant, // for animating over time, doesn't really matter when this starts
    time_of_day: f64,
    last_frame_time: Instant,
    fps: f64,
    frame_times: VecDeque<f64>,
    counter: u8,
    exit: bool, // quit when true
}

impl App {
    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        self.last_frame_time = Instant::now();
        self.frame_times = VecDeque::new();
        while !self.exit {
            let elapsed: f64 = self.start_time.elapsed().as_secs_f64();
            self.seed = elapsed * 1.0; // Adjust speed here

            let now = Instant::now();
            let frame_duration = now.duration_since(self.last_frame_time).as_secs_f64();
            self.last_frame_time = now;

            // store the frame time
            self.frame_times.push_back(frame_duration);
            // remove old frame times
            while self.frame_times.len() > 1 && self.frame_times.iter().sum::<f64>() > 10.0 {
                self.frame_times.pop_front();
            }

            // calculate the average FPS
            let total_time: f64 = self.frame_times.iter().sum();
            self.fps = self.frame_times.len() as f64 / total_time;


            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    /// updates the application's state based on user input
    fn handle_events(&mut self) -> io::Result<()> {
        if event::poll(Duration::from_millis(0))? {
            match event::read()? {
                // it's important to check that the event is a key press event as
                // crossterm also emits key release and repeat events on Windows.
                Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    self.handle_key_event(key_event)
                }
                _ => {}
            };
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Char('Q') => self.exit(),
            KeyCode::Left => { self.counter += 1},
            KeyCode::Right => self.increment_counter(),
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn increment_counter(&mut self) {
        self.counter += 1;
    }

    fn decrement_counter(&mut self) {
        self.counter -= 1;
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // let title = Line::from(" Counter App Tutorial ".bold());
        // let instructions = Line::from(vec![
        //     " Decrement ".into(),
        //     "<Left>".blue().bold(),
        //     " Increment ".into(),
        //     "<Right>".blue().bold(),
        //     " Quit ".into(),
        //     "<Q> ".blue().bold(),
        //     " Area ".into(),
        //     format!("{:?} {:?} ", area.width, area.height).yellow(),
        // ]);
        // let block = Block::bordered()
        //     .title(title.centered())
        //     .title_bottom(instructions.centered())
        //     .border_set(border::THICK);
        //
        // let counter_text = Text::from(vec![Line::from(vec![
        //     "Value: ".into(),
        //     self.counter.to_string().yellow(),
        // ])]);
        //
        // Paragraph::new(counter_text)
        //     .centered()
        //     .block(block)
        //     .render(area, buf);

        let counter_text = Text::from(vec![Line::from(vec![
            " H: ".into(),
            area.height.to_string().yellow(),
            " W: ".into(),
            area.width.to_string().yellow(),
            " FPS: ".into(),
            format!("{:.2}", self.fps).yellow(),
        ])]);

        Paragraph::new(counter_text)
            .centered()
            .render(area, buf);

        let perlin = Perlin::new(1);

        let width: f64 = area.width as f64;
        let height: f64 = area.height as f64;

        for x in (0..area.width).step_by(2) {
            for y in (0.. area.height).step_by(1) {
                let nx: f64 = x as f64 / 2.0;
                let ny: f64 = height - y as f64; // so y starts at bottom
                let seed: f64 = self.seed / 1.0;

                let mut color= (0, 0, 0); // the color of the pixel being rendered
                // let color_water_shallow = (0, 129, 218); // wave top
                // let color_water_deep = (0, 24, 47); // wave body
                let color_water_shallow = (0, 64, 110); // wave top
                let color_water_deep = (0, 12, 24); // wave body
                let color_sky_dark = (1, 0, 57); // sky top
                let color_sky_medium = (10, 3, 73); // sky mid
                let color_sky_light1 = (72, 28, 86); // sky bottom
                let color_sky_light2 = (83, 54, 116); // sky bottom
                let color_star = (255, 255, 200); // star

                // wave stuff
                let wx: f64 = nx * 200.0 / width; // wave x (scales with width)
                let speed_var: f64 = (perlin.get([wx / 5.0, seed + 20.0]) + 1.0) / 2.0; // variation in speed
                let base_speed: f64 = (perlin.get([seed / 2.0 + 10.0]) + 1.0) / 2.0;
                let speed: f64 = base_speed * (0.5 + 0.5 * speed_var); // speed of the wave
                let offset: f64 = seed + speed; // offset in time
                let amp_seed: f64 = (wx / 10.0) + offset;
                let amp: f64 = perlin.get([amp_seed, seed / 2.0]); // amplitude of the wave
                let wave_height: f64 = (0.3 * height) + (amp * 0.06 * height); // final wave height in px

                if ny < wave_height { // water
                    color = gradient(color_water_deep, color_water_shallow, (ny / wave_height) as f32);
                } else { // sky
                    let sky_height: f64 = 0.3 * height; // lowest point of the sky (in px high). basically when amp is 0
                    let mut sky_y: f64 = ((ny - sky_height) / (height - sky_height)).clamp(0.0, 1.0); // 0-1 in sky
                    let skyline = 0.7; // where the mid-gradient is (0-1)

                    // slight variation in sky color height
                    let sky_height_var: f64 = (perlin.get([nx / 20.0, seed / 10.0]) + 1.0) / 2.0;
                    sky_y = sky_y - (sky_height_var * 0.1);

                    if sky_y <= skyline {
                        // lower sky

                        // slight variation in sky color
                        let mut sky_color_var: f64 = (perlin.get([nx / 20.0, seed / 10.0]) + 1.0) / 2.0; // same as height var for now
                        sky_color_var = sky_color_var.powf(3.0) * 0.3;
                        let color_sky_light = gradient(color_sky_light1, color_sky_light2, sky_color_var as f32);

                        let factor: f64 = (sky_y / skyline).clamp(0.0, 1.0);
                        color = gradient(color_sky_light, color_sky_medium, factor as f32)
                    } else {
                        // higher sky
                        let mut factor: f64 = ((sky_y - skyline) / (1.0 - skyline)).clamp(0.0, 1.0);
                        color = gradient(color_sky_medium, color_sky_dark, factor as f32)
                    }

                    // stars
                    // let star_chance = perlin.get([nx / 5.0, ny / 5.0, self.seed as f64]) + 1.0; // Perlin noise for star placement
                    // if star_chance > 1.5 { // Threshold for stars (adjust this value for density)
                    //     let star_brightness = (perlin.get([nx, ny, self.seed as f64 / 10.0]) + 1.0) / 2.0; // Perlin noise for brightness
                    //     let twinkle = 0.5 + 0.5 * (perlin.get([self.seed as f64 / 2.0]) * 2.0).sin(); // Twinkling effect
                    //     let brightness = (star_brightness * twinkle * 255.0) as u8;
                    //
                    //     // Override sky color with star
                    //     color = (
                    //         (color_star.0 as u16 * brightness as u16 / 255).clamp(0, 255) as u8,
                    //         (color_star.1 as u16 * brightness as u16 / 255).clamp(0, 255) as u8,
                    //         (color_star.2 as u16 * brightness as u16 / 255).clamp(0, 255) as u8,
                    //     );
                    // }
                }

                if ny == wave_height.ceil() { // foam
                    let mut foam_brightness: f64 = 20.0 + 40.0 * (perlin.get([nx / 5.0, seed + 30.0]) + 1.0) / 2.0;
                    foam_brightness = foam_brightness - 10.0;
                    color = ((color_water_shallow.0 as u16 + foam_brightness as u16).clamp(0,255) as u8,
                             (color_water_shallow.1 as u16 + foam_brightness as u16).clamp(0,255) as u8,
                             (color_water_shallow.2 as u16 + foam_brightness as u16).clamp(0,255) as u8);
                }

                let color = Color::Rgb(color.0, color.1, color.2);

                let pixel_rect = Rect {
                    x: x as u16,
                    y: y as u16,
                    width: 2,
                    height: 1,
                };

                let pixel = Block::default().style(ratatui::style::Style::default().bg(color));
                pixel.render(pixel_rect, buf);
            }
        }
    }
}

/// Interpolates between two colors (start and end) based on a gradient factor (0.0 to 1.0).
/// Colors are represented as (r, g, b) tuples, where r, g, b are u8 (0-255).
fn gradient(
    start: (u8, u8, u8), // Start color as (r, g, b)
    end: (u8, u8, u8),   // End color as (r, g, b)
    factor: f32          // Gradient factor between 0.0 and 1.0
) -> (u8, u8, u8) {
    // Clamp factor to ensure it's in the 0.0 to 1.0 range
    let factor = factor.clamp(0.0, 1.0);

    // Linearly interpolate each color channel
    let r = start.0 as f32 + (end.0 as f32 - start.0 as f32) * factor;
    let g = start.1 as f32 + (end.1 as f32 - start.1 as f32) * factor;
    let b = start.2 as f32 + (end.2 as f32 - start.2 as f32) * factor;

    // Convert back to u8 and return as a tuple
    (r.round() as u8, g.round() as u8, b.round() as u8)
}

fn hue_shift(rgb: (u8, u8, u8), shift: f32) -> (u8, u8, u8) {
    let (mut h, s, v) = rgb_to_hsv(rgb);
    h = (h + shift) % 1.0;
    hsv_to_rgb((h, s, v))
}

fn rgb_to_hsv((r, g, b): (u8, u8, u8)) -> (f32, f32, f32) {
    let r = (r as f32 / 255.0);
    let g = (g as f32 / 255.0);
    let b = (b as f32 / 255.0);

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;

    let h = if delta == 0.0 {
        0.0
    } else if max == r {
        ((g - b) / delta) % 6.0
    } else if max == g {
        (b - r) / delta + 2.0
    } else {
        (r - g) / delta + 4.0
    } / 6.0;
    let h = if h < 0.0 { h + 1.0 } else { h };
    let s = if max == 0.0 { 0.0 } else { delta / max };
    (h, s, max)
}

fn hsv_to_rgb((h, s, v): (f32, f32, f32)) -> (u8, u8, u8) {
    let h = h * 6.0;
    let i = h.floor() as u8;
    let f = h - h.floor();

    let p = v * (1.0 - s);
    let q = v * (1.0 - f * s);
    let t = v * (1.0 - (1.0 - f) * s);

    let (r, g, b) = match i % 6 {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        5 => (v, p, q),
        _ => (0.0, 0.0, 0.0),
    };

    (
        (r * 255.0).round() as u8,
        (g * 255.0).round() as u8,
        (b * 255.0).round() as u8,
    )
}