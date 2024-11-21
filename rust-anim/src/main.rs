use std::collections::{VecDeque, HashMap};
use std::io;
use std::time::{Duration, Instant};
use chrono::{Local, Timelike};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    text::{Line, Text},
    widgets::{Paragraph, Widget},
    DefaultTerminal, Frame,
};
use ratatui::style::Color;
use noise::{NoiseFn, Perlin};

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();

    let colors = HashMap::from([
        ("sky:early_morning", (20, 20, 60)),
        ("sky:sunrise", (255, 100, 50)),
        ("sky:morning", (135, 206, 250)),
        ("sky:midday", (135, 206, 255)),
        ("sky:afternoon", (255, 250, 205)),
        ("sky:sunset", (255, 69, 0)),
        ("sky:evening", (75, 0, 130)),
        ("sky:night", (10, 10, 30)),
        ("water:shallow", (0, 129, 218)),
        ("water:deep", (0, 24, 47)),
        // ("water:shallow", (0, 64, 110)), // wave top
        // ("water:deep", (0, 12, 24)), // wave body
        ("sky:dark", (1, 0, 57)), // sky top
        ("sky:medium", (10, 3, 73)), // sky mid
        ("sky:light1", (72, 28, 86)), // sky bottom
        ("sky:light2", (83, 54, 116)), // sky bottom
    // let color_star = (255, 255, 200); // star
    ]);

    let app_result = App {
        seed: 0.0,
        start_time: Instant::now(),
        time_of_day: 0.0,
        stage: "limbo",
        stage_progress: 0.0,
        last_frame_time: Instant::now(),
        fps: 0.0,
        frame_times: VecDeque::new(),
        perlin: Perlin::new(1),
        colors,
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
    time_of_day: f64, // seconds since midnight (including ms)
    stage: &'static str, // current stage of the day
    stage_progress: f64, // progress through the current stage (0.0 to 1.0)
    last_frame_time: Instant,
    fps: f64,
    frame_times: VecDeque<f64>,
    perlin: Perlin,
    colors: HashMap<&'static str, (u8, u8, u8)>, // hashmap of all color constants
    exit: bool, // quit when true
}

impl App {
    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        self.last_frame_time = Instant::now();
        self.frame_times = VecDeque::new();
        while !self.exit {
            let elapsed: f64 = self.start_time.elapsed().as_secs_f64();
            self.seed = elapsed * 1.0; // adjust speed

            let now = Instant::now();
            let frame_duration = now.duration_since(self.last_frame_time).as_secs_f64();
            self.last_frame_time = now;

            // store the frame time
            self.frame_times.push_back(frame_duration);
            // remove old frame times
            while self.frame_times.len() > 1 && self.frame_times.iter().sum::<f64>() > 3.0 {
                self.frame_times.pop_front();
            }

            // calculate the average FPS
            let total_time: f64 = self.frame_times.iter().sum();
            self.fps = self.frame_times.len() as f64 / total_time;

            let now_local = Local::now().time();
            // total time since midnight in seconds (with decimal)
            self.time_of_day =
                now_local.hour() as f64 * 3600.0 +
                now_local.minute() as f64 * 60.0 +
                now_local.second() as f64 +
                now_local.nanosecond() as f64 / 1_000_000_000.0;

            match self.time_of_day {
                0.0..=18_000.0 => {
                    self.stage = "night";
                    self.stage_progress = self.time_of_day / 18_000.0; },
                18_000.0..=21_600.0 => { // 3600
                    self.stage = "sunrise";
                    self.stage_progress = (self.time_of_day - 18_000.0) / 18_000.0; },
                21_600.0..=43_200.0 => { // 21600
                    self.stage = "morning";
                    self.stage_progress = self.time_of_day / 18_000.0; },
                43_200.0..=54_000.0 => {// 3600
                    self.stage = "midday";
                    self.stage_progress = self.time_of_day / 18_000.0; },
                54_000.0..=64_800.0 => {// 3600
                    self.stage = "afternoon";
                    self.stage_progress = self.time_of_day / 18_000.0; },
                64_800.0..=68_400.0 => {// 3600
                    self.stage = "sunset";
                    self.stage_progress = self.time_of_day / 18_000.0; },
                68_400.0..=76_800.0 => {// 3600
                    self.stage = "evening";
                    self.stage_progress = self.time_of_day / 18_000.0; },
                76_800.0..=86_400.0 => {// 3600
                    self.stage = "night";
                    self.stage_progress = self.time_of_day / 18_000.0; },
                _ => println!("limbo"), // Anything outside those ranges
            }

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
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
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
            " Time: ".into(),
            format!("{:.2}", self.time_of_day).yellow(),
        ])]);

        Paragraph::new(counter_text)
            .centered()
            .render(area, buf);

        let block_style = ratatui::style::Style::default();

        let defcol: (u8, u8, u8) = (0, 0, 0);

        let width: f64 = area.width as f64;
        let height: f64 = area.height as f64;

        for x in (0..area.width).step_by(2) {
            for y in (0.. area.height).step_by(1) {
                let nx: f64 = x as f64 / 2.0;
                let ny: f64 = height - y as f64; // so y starts at bottom
                let seed: f64 = self.seed / 1.0;

                let mut color = (0, 0, 0); // the color of the pixel being rendered

                // water stuff
                // TODO: change water stuff based on random weather?
                let wx: f64 = nx * 200.0 / width; // water x (scales with width)
                let speed_var: f64 = (self.perlin.get([wx / 5.0, seed + 20.0]) + 1.0) / 2.0; // variation in speed
                let base_speed: f64 = (self.perlin.get([seed / 2.0 + 10.0]) + 1.0) / 2.0;
                let speed: f64 = base_speed * (0.5 + 0.5 * speed_var); // speed of the waves
                let offset: f64 = seed + speed; // offset in time
                let amp_seed: f64 = (wx / 10.0) + offset;
                let amp: f64 = self.perlin.get([amp_seed, seed / 2.0]); // amplitude of the waves
                let water_height: f64 = (0.3 * height) + (amp * 0.06 * height); // final water height in px

                let is_water: bool = ny < water_height; // is this pixel water? else sky
                if is_water { // water
                    color = gradient(
                        *self.colors.get("water:deep").unwrap_or(&defcol),
                        *self.colors.get("water:shallow").unwrap_or(&defcol),
                        ny / water_height
                    );
                } else { // sky
                    let sky_height: f64 = 0.3 * height; // lowest point of the sky (in px high). basically when amp is 0
                    let mut sky_y: f64 = ((ny - sky_height) / (height - sky_height)).clamp(0.0, 1.0); // 0-1 in sky
                    let skyline = 0.7; // where the mid-gradient is (0-1)

                    // slight variation in sky color height
                    let sky_height_var: f64 = (self.perlin.get([nx / 20.0, seed / 10.0]) + 1.0) / 2.0;
                    sky_y = sky_y - (sky_height_var * 0.1);

                    if sky_y <= skyline {
                        // lower sky

                        // slight variation in sky color
                        let mut sky_color_var: f64 = (self.perlin.get([nx / 20.0, seed / 10.0]) + 1.0) / 2.0; // same as height var for now
                        sky_color_var = sky_color_var.powf(3.0) * 0.3;
                        let color_sky_light = gradient(
                            *self.colors.get("sky:light1").unwrap_or(&defcol),
                            *self.colors.get("sky:light2").unwrap_or(&defcol),
                            sky_color_var
                        );

                        let factor: f64 = (sky_y / skyline).clamp(0.0, 1.0);
                        color = gradient(
                            color_sky_light,
                            *self.colors.get("sky:medium").unwrap_or(&defcol),
                            factor
                        );
                    } else {
                        // higher sky
                        let factor: f64 = ((sky_y - skyline) / (1.0 - skyline)).clamp(0.0, 1.0);
                        color = gradient(
                            *self.colors.get("sky:medium").unwrap_or(&defcol),
                            *self.colors.get("sky:dark").unwrap_or(&defcol),
                            factor
                        );
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

                if ny == water_height.ceil() { // foam
                    let mut foam_brightness: f64 = 20.0 + 40.0 * (self.perlin.get([nx / 5.0, seed + 30.0]) + 1.0) / 2.0;
                    foam_brightness = foam_brightness - 10.0; // TODO: this is at night. 0 for day?
                    color = (((*self.colors.get("water:shallow").unwrap_or(&defcol)).0 as u16 + foam_brightness as u16).clamp(0,255) as u8,
                             ((*self.colors.get("water:shallow").unwrap_or(&defcol)).1 as u16 + foam_brightness as u16).clamp(0,255) as u8,
                             ((*self.colors.get("water:shallow").unwrap_or(&defcol)).2 as u16 + foam_brightness as u16).clamp(0,255) as u8
                    );
                }

                let color = Color::Rgb(color.0, color.1, color.2);

                let pixel_rect = Rect {
                    x,
                    y,
                    width: 2,
                    height: 1,
                };

                buf.set_style(pixel_rect, block_style.bg(color));
            }
        }
    }
}

// interpolates between two colors (start and end) based on a gradient factor (0.0 to 1.0).
// TODO: different modes?
fn gradient(start: (u8, u8, u8), end: (u8, u8, u8), factor: f64) -> (u8, u8, u8) {
    // Clamp factor to ensure it's in the 0.0 to 1.0 range
    let factor = factor.clamp(0.0, 1.0);

    // Linearly interpolate each color channel
    let r = start.0 as f64 + (end.0 as f64 - start.0 as f64) * factor;
    let g = start.1 as f64 + (end.1 as f64 - start.1 as f64) * factor;
    let b = start.2 as f64 + (end.2 as f64 - start.2 as f64) * factor;

    // Convert back to u8 and return as a tuple
    (r.round() as u8, g.round() as u8, b.round() as u8)
}

// // shifts the hue of an RGB color by a given amount (0.0 to 1.0)
// fn hue_shift(rgb: (u8, u8, u8), shift: f64) -> (u8, u8, u8) {
//     let (mut h, s, v) = rgb_to_hsv(rgb);
//     h = (h + shift) % 1.0;
//     hsv_to_rgb((h, s, v))
// }
//
// // shifts the saturation of an RGB color by a given amount (0.0 to 1.0)
// fn saturation_shift(rgb: (u8, u8, u8), shift: f64) -> (u8, u8, u8) {
//     let (h, mut s, v) = rgb_to_hsv(rgb);
//     s = (s + shift).clamp(0.0, 1.0);
//     hsv_to_rgb((h, s, v))
// }
//
// // shifts the value (brightness) of an RGB color by a given amount (0.0 to 1.0)
// fn value_shift(rgb: (u8, u8, u8), shift: f64) -> (u8, u8, u8) {
//     let (h, s, mut v) = rgb_to_hsv(rgb);
//     v = (v + shift).clamp(0.0, 1.0);
//     hsv_to_rgb((h, s, v))
// }
//
// // converts an RGB color to HSV
// fn rgb_to_hsv((r, g, b): (u8, u8, u8)) -> (f64, f64, f64) {
//     let r = (r as f64 / 255.0);
//     let g = (g as f64 / 255.0);
//     let b = (b as f64 / 255.0);
//
//     let max = r.max(g).max(b);
//     let min = r.min(g).min(b);
//     let delta = max - min;
//
//     let h = if delta == 0.0 {
//         0.0
//     } else if max == r {
//         ((g - b) / delta) % 6.0
//     } else if max == g {
//         (b - r) / delta + 2.0
//     } else {
//         (r - g) / delta + 4.0
//     } / 6.0;
//     let h = if h < 0.0 { h + 1.0 } else { h };
//     let s = if max == 0.0 { 0.0 } else { delta / max };
//     (h, s, max)
// }
//
// // converts an HSV color to RGB
// fn hsv_to_rgb((h, s, v): (f64, f64, f64)) -> (u8, u8, u8) {
//     let h = h * 6.0;
//     let i = h.floor() as u8;
//     let f = h - h.floor();
//
//     let p = v * (1.0 - s);
//     let q = v * (1.0 - f * s);
//     let t = v * (1.0 - (1.0 - f) * s);
//
//     let (r, g, b) = match i % 6 {
//         0 => (v, t, p),
//         1 => (q, v, p),
//         2 => (p, v, t),
//         3 => (p, q, v),
//         4 => (t, p, v),
//         5 => (v, p, q),
//         _ => (0.0, 0.0, 0.0),
//     };
//
//     (
//         (r * 255.0).round() as u8,
//         (g * 255.0).round() as u8,
//         (b * 255.0).round() as u8,
//     )
// }