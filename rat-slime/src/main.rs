use std::collections::{HashMap, VecDeque};
use color_eyre::Result;
use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    layout::{Constraint, Layout, Position},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, List, ListItem, Paragraph, Borders},
    DefaultTerminal, Frame,
};
use std::fs::File;
use std::io;
use std::io::BufReader;
use std::time::{Duration, Instant};
use crossterm::event::KeyEvent;
use ratatui::layout::{Alignment, Direction, Rect};
use ratatui::symbols::border;
use rodio::{Decoder, OutputStream, Sink};
use rodio::source::{ChannelVolume, SineWave, Source};

fn main() -> Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = App::new().run(terminal);
    ratatui::restore();
    Ok(result?)
}

/// struct to hold information for a song/audio
struct Song {
    source: Box<dyn Source<Item = f32> + Send>,
    sample_rate: u32,
    channels: u16,
    duration: f64,
}

impl Song {
    fn new(source: impl Source<Item = f32> + Send + 'static) -> Self {
        let sample_rate = source.sample_rate();
        let channels = source.channels();
        let duration = source.duration().unwrap_or(0.0);
        Self {
            source: Box::new(source),
            sample_rate,
            channels,
            duration,
        }
    }
}

struct Player {
    /// Sink for the audio
    sink: Sink,
    /// Playback speed, 1.0 is normal speed
    playback_speed: f32,
    /// Current volume (0-1)
    volume: f32,
    /// Current position in the song, in ms
    position: f64,
    /// Total duration of the song, in ms
    duration: f64,
    /// Whether the audio is playing or paused
    playing: bool,
    /// Metadata of the song
    audio_metadata: HashMap<String, String>,
    /// Loop type
    loop_type: LoopType,
    /// Whether the playlist is shuffled
    shuffle: bool,
}

impl Player {
    fn set_playback_speed(&mut self, speed: f32) {
        let speed = speed.max(0.5).min(2.0);
        self.playback_speed = speed;
        self.sink.set_speed(speed);
    }
    fn get_playback_speed(&self) -> f32 {
        self.playback_speed
    }
    fn set_volume(&mut self, volume: f32) {
        let volume = volume.max(0.0).min(1.0);
        self.volume = volume;
        self.sink.set_volume(volume);
    }
    fn get_volume(&self) -> f32 {
        self.volume
    }
    fn set_position(&mut self, position: f64) {
        self.position = position;
    }
    fn get_position(&self) -> f64 {
        self.position
    }
    fn set_duration(&mut self, duration: f64) {
        self.duration = duration;
    }
    fn get_duration(&self) -> f64 {
        self.duration
    }
    /// Toggles the playback status of the Player
    fn toggle_playing(&mut self) {
        if self.playing {
            self.pause();
        } else {
            self.play();
        }
    }
    /// Pauses the Player
    fn pause(&mut self) {
        self.playing = false;
        self.sink.pause();
    }
    /// Resumes the Player
    fn play(&mut self) {
        self.playing = true;
        self.sink.play();
    }
    fn set_playing(&mut self, playing: bool) {
        self.playing = playing;
    }
    fn is_playing(&self) -> bool {
        self.playing
    }
}

enum LoopType {
    None, // No loop
    Loop, // Loop the playlist
    LoopOne // Loop the current song
}

struct App {
    _stream: OutputStream,
    player: Player,
    start_time: Instant,
    last_frame_time: Instant,
    fps: f64,
    frame_times: VecDeque<f64>,
    exit: bool, // quit when true
}

impl App {
    fn new() -> Self {
        // _stream must live as long as the sink
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();

        let file = BufReader::new(File::open("audio.mp3").unwrap());
        // Decode that sound file into a source
        let source = Decoder::new(file).unwrap();
        sink.append(source);

        sink.set_speed(1.0);
        sink.set_volume(0.2);
        sink.play();

        Self {
            _stream,
            // init player with default settings
            player: Player {
                sink,
                playback_speed: 1.0,
                volume: 0.2,
                position: 0.0,
                duration: 0.0,
                playing: true,
                audio_metadata: HashMap::new(),
                loop_type: LoopType::None,
                shuffle: false,
            },
            start_time: Instant::now(),
            last_frame_time: Instant::now(),
            fps: 0.0,
            frame_times: VecDeque::new(),
            exit: false,
        }
    }

    pub fn run(mut self, mut terminal: DefaultTerminal) -> io::Result<()> {
        self.last_frame_time = Instant::now();
        self.frame_times = VecDeque::new();
        while !self.exit {
            let elapsed: f64 = self.start_time.elapsed().as_secs_f64();
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

            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
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
            KeyCode::Char('q') | KeyCode::Backspace | KeyCode::Esc => {
                self.exit();
            }
            KeyCode::Char(' ') => {
                self.player.toggle_playing();
            }
            KeyCode::Up => {
                let volume = self.player.get_volume() + 0.1;
                self.player.set_volume(volume);
            }
            KeyCode::Down => {
                let volume = self.player.get_volume() - 0.1;
                self.player.set_volume(volume);
            }
            KeyCode::Char('o') => {
                let speed = self.player.get_playback_speed() - 0.1;
                self.player.set_playback_speed(speed);
            }
            KeyCode::Char('p') => {
                let speed = self.player.get_playback_speed() + 0.1;
                self.player.set_playback_speed(speed);
            }
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn draw(&self, frame: &mut Frame) {
        if (frame.area().width < 130) || (frame.area().height < 40) {
            let area = frame.area();
            let text = "Please expand your terminal or zoom out!";
            let text_height = 1;
            let text_width = text.len() as u16;

            // calculate the centered position for the box
            let box_width = text_width + 4;
            let box_height = text_height + 2;
            let x = (area.width.saturating_sub(box_width)) / 2;
            let y = (area.height.saturating_sub(box_height)) / 2;

            // create the centered box
            let box_area = Rect::new(x, y, box_width, box_height);
            frame.render_widget(Block::bordered(), box_area);

            // render the centered text inside the box
            frame.render_widget(
                Paragraph::new(text)
                    .alignment(Alignment::Center)
                    .style(Style::default().fg(Color::White)),
                Rect::new(x + 1, y + 1, text_width + 2, text_height),
            );
            return;
        }

        // create the main block that takes the entire terminal size
        let main = Rect::new(
            0,
            0,
            frame.area().width,
            frame.area().height,
        );


        let main_title = Line::from(vec![
            Span::raw(" tylersfoot's audio player | "),
            Span::raw("HxW: "),
            Span::styled(format!("{}x{}", main.width, main.height), Style::default().fg(Color::Yellow)),
            Span::raw(" | FPS: "),
            Span::styled(format!("{:.0} ", self.fps), Style::default().fg(Color::Yellow)),
        ]);

        frame.render_widget(
            Block::default()
                .title(main_title)
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::White)),
            main,
        );

        let main_pad = Rect::new(
            main.x + 2,
            main.y + 1,
            main.width.saturating_sub(4),
            main.height.saturating_sub(2),
        );


        let [block_top, block_visualizer] = Layout::vertical([
            Constraint::Length(35),
            Constraint::Min(1),
        ]).areas(main_pad);

        frame.render_widget(Block::bordered()
                                .title(" Visualizer ")
                                .title_alignment(Alignment::Center), block_visualizer);

        let [block_left, block_player, block_right] = Layout::horizontal([
            Constraint::Min(1),
            Constraint::Length(50),
            Constraint::Min(1),
        ]).areas(block_top);

        frame.render_widget(
            // multiline text
            self.player_content(block_player.width as usize, block_player.height as usize)
                .block(default_block(" Player ")),
            block_player,
        );

        let [block_file_picker, block_queue] = Layout::vertical([
            Constraint::Percentage(20),
            Constraint::Percentage(80),
        ]).areas(block_left);
        frame.render_widget(Block::bordered()
                                .title(" File Picker ")
                                .title_alignment(Alignment::Center), block_file_picker);
        frame.render_widget(Block::bordered()
                                .title(" Queue ")
                                .title_alignment(Alignment::Center), block_queue);

        let [block_telly, block_osc] = Layout::vertical([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ]).areas(block_right);

        frame.render_widget(Block::bordered()
                                .title(" Oscilloscope ")
                                .title_alignment(Alignment::Center), block_osc);

        let sine_wave = self.generate_sine_wave(
            block_telly.width as usize - 2,
            block_telly.height as usize - 2
        );

        frame.render_widget(
            Paragraph::new(sine_wave)
                .alignment(Alignment::Left)
                .style(Style::default().fg(Color::White))
                .block(default_block(" Meow ")),
            block_telly,
        );

    }

    fn player_content(&self, width: usize, height: usize) -> Paragraph {

        let volume = format!("{:.2} ", self.player.get_volume());
        let playing = self.player.is_playing().to_string();
        let speed = format!("{:.2} ", self.player.get_playback_speed());

        let content_lines: Vec<Line> = vec![
            // Line::from(Span::raw("test")),
            Line::from(Span::styled("meow hi umm meow", Style::default().fg(Color::White))),
            Line::from(Span::styled("d-_-b", Style::default().fg(Color::LightMagenta))),
            Line::from(Span::raw(" ")),
            // Line::from(Span::raw(self.player.sink.source().sample_rate().to_string())),
            Line::from(Span::raw(" ")),
            Line::from(vec![
                Span::styled("playing: ", Style::default().fg(Color::White)),
                Span::styled(playing, Style::default().fg(Color::Yellow)),
            ]),
            Line::from(vec![
                Span::styled("volume: ", Style::default().fg(Color::White)),
                Span::styled(volume, Style::default().fg(Color::Yellow)),
            ]),
            Line::from(vec![
                Span::styled("speed: ", Style::default().fg(Color::White)),
                Span::styled(speed, Style::default().fg(Color::Yellow)),
            ]),
        ];

        let content_height = content_lines.len();
        let padding_top = if height > content_height {
            (height - content_height) / 2
        } else {0};

        let mut lines = Vec::new();
        for _ in 0..padding_top {
            lines.push(Line::from(""));
        }
        lines.extend(content_lines);

        Paragraph::new(Text { lines, ..Default::default() })
            .alignment(Alignment::Center)
    }

    fn generate_sine_wave(&self, width: usize, height: usize) -> String {
        let time = self.start_time.elapsed().as_secs_f64() * 1.5;
        let mut wave = String::new();
        let frequency = 1.0 * std::f64::consts::PI / width as f64;

        for y in 0..height {
            for x in 0..width {
                // let taper_factor = 1.0 - (x as f64 / width as f64 * 2.0 - 1.0).abs();
                // let value = (x as f64 * frequency + time).sin() * taper_factor;
                let value = (x as f64 * frequency + time).sin();
                let scaled_value = ((value + 1.0) * (height as f64 / 2.3)).round() as usize;
                if scaled_value == y {
                    wave.push('#');
                } else {
                    wave.push(' ');
                }
            }
            wave.push('\n');
        }
        wave
    }
}

fn default_block(title: &str) -> Block {
    Block::default()
        .borders(Borders::ALL)
        .title(title)
        .title_alignment(Alignment::Center)
}