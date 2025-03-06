//! # [Ratatui] User Input example
//!
//! The latest version of this example is available in the [examples] folder in the repository.
//!
//! Please note that the examples are designed to be run against the `main` branch of the Github
//! repository. This means that you may not be able to compile with the latest release version on
//! crates.io, or the one that you have installed locally.
//!
//! See the [examples readme] for more information on finding examples that match the version of the
//! library you are using.
//!
//! [Ratatui]: https://github.com/ratatui/ratatui
//! [examples]: https://github.com/ratatui/ratatui/blob/main/examples
//! [examples readme]: https://github.com/ratatui/ratatui/blob/main/examples/README.md

// A simple example demonstrating how to handle user input. This is a bit out of the scope of
// the library as it does not provide any input handling out of the box. However, it may helps
// some to get started.
//
// This is a very simple example:
//   * An input box always focused. Every character you type is registered here.
//   * An entered character is inserted at the cursor position.
//   * Pressing Backspace erases the left character before the cursor position
//   * Pressing Enter pushes the current input in the history of previous messages. **Note: ** as
//   this is a relatively simple example unicode characters are unsupported and their use will
// result in undefined behaviour.
//
// See also https://github.com/rhysd/tui-textarea and https://github.com/sayanarijit/tui-input/

use std::collections::HashMap;
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
use std::io::BufReader;
use std::time::Duration;
use ratatui::layout::{Alignment, Direction};
use ratatui::symbols::border;
use rodio::{Decoder, OutputStream, Sink};
use rodio::source::{SineWave, Source};

fn main() -> Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let app_result = App::new().run(terminal);
    ratatui::restore();
    app_result
}

struct Player {
    /// Sink for the audio
    sink: Sink,
    /// Playback speed, in percent (25-400)
    playback_speed: f32,
    /// Current volume, in percent (0-100)
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
        self.playback_speed = speed;
    }
    fn get_playback_speed(&self) -> f32 {
        self.playback_speed
    }
    fn set_volume(&mut self, volume: f32) {
        self.volume = volume;
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

        Self {
            _stream,
            // init player with default settings
            player: Player {
                sink,
                playback_speed: 1.0,
                volume: 100.0,
                position: 0.0,
                duration: 0.0,
                playing: false,
                audio_metadata: HashMap::new(),
                loop_type: LoopType::None,
                shuffle: false,
            },
        }
    }

    fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        loop {
            terminal.draw(|frame| self.draw(frame))?;

            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Backspace | KeyCode::Esc => {
                            return Ok(());
                        }
                        KeyCode::Char(' ') => {
                            self.player.toggle_playing();
                        }
                        // KeyCode::Enter => self.submit_message(),
                        // KeyCode::Char(to_insert) => self.enter_char(to_insert),
                        // KeyCode::Left => self.move_cursor_left(),
                        // KeyCode::Right => self.move_cursor_right(),
                        _ => {}
                    }
                }
            }
        }
    }

    fn draw(&self, frame: &mut Frame) {
        let size = frame.area();

        // Create the main block that takes the entire terminal size
        let main_block = Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White));
        frame.render_widget(main_block, size);

        // Calculate the centered position for the player block
        let player_width = 40;
        let player_height = 35;
        let x = (size.width.saturating_sub(player_width)) / 2;
        let y = (size.height.saturating_sub(player_height)) / 2;

        // Create the player block with a fixed size of 100x50
        let player_block = Block::bordered()
            .title(Line::from(" Player ".bold()).centered())
            .border_set(border::THICK);
        let player_area = ratatui::layout::Rect::new(x, y, player_width, player_height);
        frame.render_widget(player_block, player_area);
    }
}