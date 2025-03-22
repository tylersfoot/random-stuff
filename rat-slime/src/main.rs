use std::collections::{VecDeque};
use color_eyre::Result;
use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    layout::{Constraint, Layout},
    style::{Color, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, Paragraph, Borders},
    DefaultTerminal, Frame,
};
use std::fs::File;
use std::{fs, io};
use std::error::Error;
use std::io::BufReader;
use std::time::{Duration, Instant};
use crossterm::event::KeyEvent;
use ratatui::layout::{Alignment, Rect};
use rodio::{Decoder, OutputStream, Sink};
use rodio::source::{Source};
use id3::{TagLike};
use lofty::file::{AudioFile, TaggedFileExt};
use lofty::prelude::Accessor;
use lofty::read_from_path;

fn main() -> Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = App::new().run(terminal);
    ratatui::restore();
    Ok(result?)
}

/// struct to hold information for a song/audio
#[derive(Clone)]
struct Song {
    /// Path to the audio file
    path: std::path::PathBuf,
    /// Sample rate of the audio
    sample_rate: u32,
    /// Number of channels in the audio (1 for mono, 2 for stereo)
    channels: u16,
    /// Duration of the audio
    duration: Duration,
    /// Title/name of the song
    title: Option<String>,
    /// Artist of the song
    artist: Option<String>,
    /// Album of the song
    album: Option<String>,
    /// Year the song was released
    year: Option<String>,
    // /// Album cover of the song
    // cover: Option<Vec<u8>>,
}

impl Song {
    fn new(path: std::path::PathBuf) -> Self {
        let file = File::open(&path).unwrap();
        let buf_reader = BufReader::new(file);
        let source = Decoder::new(buf_reader).unwrap();
        let sample_rate = source.sample_rate();
        let channels = source.channels();
        let mut song = Self {
            path,
            sample_rate,
            channels,
            duration: Duration::from_secs(0),
            title: None,
            artist: None,
            album: None,
            year: None,
        };
        song.parse_metadata().expect("Metadata parsing failed");
        song
    }

    fn parse_metadata(&mut self) -> Result<(), Box<dyn Error>> {
        let tagged_file = read_from_path(&self.path)?;
        let tag = match tagged_file.primary_tag() {
            Some(primary_tag) => primary_tag,
            None => tagged_file.first_tag().expect("ERROR: No tags found!"),
        };
        let properties = tagged_file.properties();
        self.duration = properties.duration();

        self.title = Some(tag.title().as_deref().unwrap_or("None").to_string());
        self.artist = Some(tag.artist().as_deref().unwrap_or("None").to_string());
        self.album = Some(tag.album().as_deref().unwrap_or("None").to_string());
        self.year = Some(tag.year().unwrap_or(0).to_string());

        Ok(())
    }
    fn create_source(&self) -> Decoder<BufReader<File>> {
        let file = File::open(&self.path).unwrap();
        let buf_reader = BufReader::new(file);
        Decoder::new(buf_reader).unwrap()
    }
    fn get_filename(&self) -> &str {
        self.path.file_name().unwrap().to_str().unwrap()
    }
    fn get_title(&self) -> Option<&str> {
        self.title.as_deref()
    }
    fn get_artist(&self) -> Option<&str> {
        self.artist.as_deref()
    }
    fn get_album(&self) -> Option<&str> {
        self.album.as_deref()
    }
    fn get_year(&self) -> Option<&str> {
        self.year.as_deref()
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
    /// Whether the audio is playing or paused
    playing: bool,
    /// Loop type
    loop_type: LoopType,
    /// Whether the playlist is shuffled
    shuffle: bool,
    /// Directory of the current playlist/folder
    folder_dir: String,
    /// Queue of songs to play
    queue: VecDeque<Song>,
    /// Current song being played
    current_song: Option<Song>,
}

impl Player {
    /// Checks if the current song has finished playing
    fn update_current_song(&mut self) {
        // check if the sink is empty (song finished)
        if self.sink.empty() {
            match self.loop_type {
                LoopType::None => {
                    // if queue is empty, stop playing
                    if self.queue.is_empty() {
                        self.playing = false;
                        self.sink.stop();
                        self.current_song = None;
                        return;
                    } else {
                        // load next song
                        if let Some(next_song) = self.queue.pop_front() {
                            self.current_song = Some(next_song);
                            if let Some(ref song) = self.current_song {
                                self.sink.append(song.create_source());
                            }
                            self.on_song_change();
                        }
                        self.on_song_change();
                    }
                }

                LoopType::LoopOne => {
                    // load the same song again
                    if let Some(ref song) = self.current_song {
                        self.sink.append(song.create_source());
                    }
                    self.on_song_change();
                }

                LoopType::Loop => {
                    // add current song to the end of the queue
                    if let Some(ref song) = self.current_song {
                        self.queue.push_back(song.clone());
                    }
                    // load next song
                    if let Some(next_song) = self.queue.pop_front() {
                        self.current_song = Some(next_song);
                        if let Some(ref song) = self.current_song {
                            self.sink.append(song.create_source());
                        }
                        self.on_song_change();
                    }
                }
            }

            // if queue is not empty, load next song

        }
    }
    /// A callback that gets executed when the song changes.
    fn on_song_change(&self) {
        if let Some(song) = self.get_current_song() {
            // do something? maybe?
        }
    }
    /// Returns a list of songs in the current playlist/folder
    fn get_songs_list(&self) -> Vec<Song> {
        let mut songs = Vec::new();
        for entry in fs::read_dir(&self.folder_dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("mp3") {
                songs.push(Song::new(path));
            }
        }
        songs
    }
    fn skip(&mut self) {
        self.sink.stop();
    }
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
    fn get_duration(&self) -> f64 {
        self.current_song.as_ref().map_or(0.0, |song| song.duration.as_secs_f64())
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
    fn get_current_song(&self) -> Option<&Song> {
        self.current_song.as_ref()
    }
}

enum LoopType {
    /// No loop
    None,
    /// Loop the playlist
    Loop,
    /// Loop the current song
    LoopOne,
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

        let mut app = Self {
            _stream,
            // init player with default settings
            player: Player {
                sink,
                playback_speed: 1.0,
                volume: 0.05,
                position: 0.0,
                playing: false,
                loop_type: LoopType::Loop,
                shuffle: false,
                folder_dir: String::new(),
                queue: VecDeque::new(),
                current_song: None,
            },
            start_time: Instant::now(),
            last_frame_time: Instant::now(),
            fps: 0.0,
            frame_times: VecDeque::new(),
            exit: false,
        };
        app.initialize();
        app
    }

    fn initialize(&mut self) {
        // init - subject to change
        let folder = fs::canonicalize("songs").unwrap();
        self.player.folder_dir = folder.clone().to_str().unwrap().to_string();
        let songs = self.player.get_songs_list();

        // add songs to queue
        for song in songs {
            self.player.queue.push_back(song);
        }
        // play first song
        if let Some(song) = self.player.queue.pop_front() {
            self.player.current_song = Some(song);
            if let Some(ref song) = self.player.current_song {
                self.player.sink.append(song.create_source());
            }
            self.player.on_song_change();
        }

        self.player.sink.set_speed(self.player.get_playback_speed());
        self.player.sink.set_volume(self.player.get_volume());
        self.player.sink.play();
        self.player.playing = true;
    }

    pub fn run(mut self, mut terminal: DefaultTerminal) -> io::Result<()> {
        self.last_frame_time = Instant::now();
        self.frame_times = VecDeque::new();
        while !self.exit {
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

            self.player.update_current_song();

            // song position
            self.player.position = self.player.sink.get_pos().as_secs_f64();

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
                let volume = self.player.get_volume() + 0.01;
                self.player.set_volume(volume);
            }
            KeyCode::Down => {
                let volume = self.player.get_volume() - 0.01;
                self.player.set_volume(volume);
            }
            KeyCode::Char('o') => {
                let speed = self.player.get_playback_speed() - 0.05;
                self.player.set_playback_speed(speed);
            }
            KeyCode::Char('p') => {
                let speed = self.player.get_playback_speed() + 0.05;
                self.player.set_playback_speed(speed);
            }
            KeyCode::Enter => {
                // skip
                self.player.skip();
            }
            KeyCode::Char('l') => {
                let loop_type = match self.player.loop_type {
                    LoopType::None => LoopType::Loop,
                    LoopType::Loop => LoopType::LoopOne,
                    LoopType::LoopOne => LoopType::None,
                };
                self.player.loop_type = loop_type;
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
            Span::raw(" doob audio player | "),
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
        frame.render_widget(
            self.queue_content(block_queue.width as usize, block_queue.height as usize)
                .block(default_block(" Queue ")),
            block_queue,
        );

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

    fn queue_content(&self, width: usize, height: usize) -> Paragraph {
        // loop playlist, add current song to end
        // loop song, show playing song as next

        // let mut queue = &self.player.queue;
        let mut local_queue = self.player.queue.clone();
        let mut lines = Vec::new();
        match self.player.loop_type {
            LoopType::None => {
                let pad = (width - 10) / 2;
                let line = Line::from(vec![
                    Span::styled(format!("{:pad$}{}", "", "Loop: ", pad = pad), Style::default().fg(Color::LightRed)),
                    Span::styled("None", Style::default().fg(Color::LightYellow)),
                ]);
                lines.push(line);
            }
            LoopType::Loop => {
                let pad = (width - 14) / 2;
                let line = Line::from(vec![
                    Span::styled(format!("{:pad$}{}", "", "Loop: ", pad = pad), Style::default().fg(Color::LightRed)),
                    Span::styled("Playlist", Style::default().fg(Color::LightYellow)),
                ]);
                lines.push(line);
            }
            LoopType::LoopOne => {
                let pad = (width - 10) / 2;
                let line = Line::from(vec![
                    Span::styled(format!("{:pad$}{}", "", "Loop: ", pad = pad), Style::default().fg(Color::LightRed)),
                    Span::styled("Song", Style::default().fg(Color::LightYellow)),
                ]);
                lines.push(line);

                // clear the local queue and add the current song only
                local_queue.clear();
                if let Some(song) = &self.player.current_song {
                    local_queue.push_back(song.clone());
                }
            }
        }

        let max = 22; // max num of songs to show
        for (i, song) in local_queue.iter().enumerate() {
            if i >= max {
                lines.push(Line::from(Span::styled("      ...", Style::default().fg(Color::LightCyan))));
                break;
            }

            let title = song.get_title().unwrap_or("--").to_string();
            let artist = song.get_artist().unwrap_or("--").to_string();

            if i == 0 {
                lines.push(Line::from(Span::raw(" ")));
                let line = Line::from(vec![
                    Span::styled("   Next: ", Style::default().fg(Color::LightRed)),
                    Span::styled(title, Style::default().fg(Color::LightCyan)),
                    Span::styled(" - ", Style::default().fg(Color::White)),
                    Span::styled(artist, Style::default().fg(Color::LightYellow)),
                ]);
                lines.push(line);
                lines.push(Line::from(Span::raw(" ")));
            } else {
                let line = Line::from(vec![
                    Span::styled(format!("  {:2}. ", i + 1), Style::default().fg(Color::LightRed)),
                    Span::styled(title, Style::default().fg(Color::LightCyan)),
                    Span::styled(" - ", Style::default().fg(Color::White)),
                    Span::styled(artist, Style::default().fg(Color::LightYellow)),
                ]);
                lines.push(line);
            }
        }

        Paragraph::new(Text { lines, ..Default::default() })
            .alignment(Alignment::Left)
    }

    fn player_content(&self, width: usize, height: usize) -> Paragraph {
        let position = format!("{:.2}", self.player.get_position());
        let duration = format!("{:.2}", self.player.get_duration());
        let volume = format!("{:.2}", self.player.get_volume());
        let playing = self.player.is_playing().to_string();
        let mut folder = self.player.folder_dir.to_string();
        if folder.len() >= 20 {
            folder = format!("~{}", &folder[folder.len() - 20..])
        }
        let face = if self.player.is_playing() { "d-_-b" } else { "do_ob" };
        let speed = format!("{:.2}", self.player.get_playback_speed());
        let loop_type = match self.player.loop_type {
            LoopType::None => "none",
            LoopType::Loop => "loop",
            LoopType::LoopOne => "loop_one",
        };
        let shuffle = self.player.shuffle.to_string();
        let mut sample_rate = "--".to_string();
        let mut channels = "--".to_string();
        let mut title = "--";
        let mut artist = "--";
        let mut album = "--";
        let mut year = "--";

        if self.player.current_song.is_some() {
            sample_rate = format!("{:.2}khz", self.player.current_song.as_ref().unwrap().sample_rate);
            channels = format!("{:.2}", self.player.current_song.as_ref().unwrap().channels);
            title = self.player.current_song.as_ref().unwrap().get_title().unwrap_or("--");
            artist = self.player.current_song.as_ref().unwrap().get_artist().unwrap_or("--");
            album = self.player.current_song.as_ref().unwrap().get_album().unwrap_or("--");
            year = self.player.current_song.as_ref().unwrap().get_year().unwrap_or("--");
        }

        let queue = self.player.queue.len().to_string();

        let content_lines: Vec<Line> = vec![
            // Line::from(Span::raw("test")),
            Line::from(Span::styled(face, Style::default().fg(Color::LightMagenta))),
            Line::from(Span::raw(" ")),
            Line::from(vec![
                Span::styled("position: ", Style::default().fg(Color::White)),
                Span::styled(position, Style::default().fg(Color::Yellow)),
            ]),
            Line::from(vec![
                Span::styled("duration: ", Style::default().fg(Color::White)),
                Span::styled(duration, Style::default().fg(Color::Yellow)),
            ]),
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
            Line::from(vec![
                Span::styled("loop type: ", Style::default().fg(Color::White)),
                Span::styled(loop_type, Style::default().fg(Color::Yellow)),
            ]),
            Line::from(vec![
                Span::styled("shuffle: ", Style::default().fg(Color::White)),
                Span::styled(shuffle, Style::default().fg(Color::Yellow)),
            ]),
            Line::from(vec![
                Span::styled("folder: ", Style::default().fg(Color::White)),
                Span::styled(folder, Style::default().fg(Color::Yellow)),
            ]),
            Line::from(Span::raw(" ")),
            Line::from(vec![
                Span::styled("sample rate: ", Style::default().fg(Color::White)),
                Span::styled(sample_rate, Style::default().fg(Color::Yellow)),
            ]),
            Line::from(vec![
                Span::styled("channels: ", Style::default().fg(Color::White)),
                Span::styled(channels, Style::default().fg(Color::Yellow)),
            ]),
            Line::from(Span::raw(" ")),
            Line::from(vec![
                Span::styled("title: ", Style::default().fg(Color::White)),
                Span::styled(title, Style::default().fg(Color::Yellow)),
            ]),
            Line::from(vec![
                Span::styled("artist: ", Style::default().fg(Color::White)),
                Span::styled(artist, Style::default().fg(Color::Yellow)),
            ]),
            Line::from(vec![
                Span::styled("album: ", Style::default().fg(Color::White)),
                Span::styled(album, Style::default().fg(Color::Yellow)),
            ]),
            Line::from(vec![
                Span::styled("year: ", Style::default().fg(Color::White)),
                Span::styled(year, Style::default().fg(Color::Yellow)),
            ]),
            Line::from(Span::raw(" ")),
            Line::from(vec![
                Span::styled("queue length: ", Style::default().fg(Color::White)),
                Span::styled(queue, Style::default().fg(Color::Yellow)),
            ]),
            Line::from(Span::raw("⏵ ⏸ ⏹ ⏭ ⏮ ⏴ ⏪ ⏩ 🔀 🔁 🔂 🔄")),
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

/*
todo
- limit to 175x50?
- add cool background if bigger than that
- queue right hand padding, maybe cut off name to write artist?
- add album cover

known issues
- switch loop back to playlist doesn't add old songs
- switching speed doesnt update position right
- audio stuttering???


other
⏵ ⏸ ⏹ ⏭ ⏮ ⏴ ⏪ ⏩ 🔀 🔁 🔂 🔄

 */