#![allow(unused)]

use fs_extra::dir;
use core::time;
use std::{array, iter::FlatMap, os, process::exit, time::{Duration, Instant}};
use xcap::Monitor;
use rustautogui::RustAutoGui;
use std::collections::HashMap;
use rdev::{Event, listen, EventType, Key};
use std::thread;
use std::io::{self, Write};
use rand::Rng;

/*  COLORS (for display)
closed = 4C545C (DCDCDC)
flagged = D8E0E8 (F75656)
mine = 7B7B7B (FF3333)
0/open = 384048
1 = 7CC7FF (blue)
2 = 66C266 (green)
3 = FF7788 (red)
4 = EE88FF (pink)
5 = DDAA22 (yellow)
6 = 66CCCC (teal)
7 = 888888 (gray)
8 = D0D8E0 (white)
*/

fn normalized(filename: String) -> String {
    filename.replace(['|', '\\', ':', '/'], "")
}

fn wait(millis: u64) {
    thread::sleep(Duration::from_millis(millis));
}

fn rgb_to_hex(r: u8, g: u8, b: u8) -> String {
    format!("{r:02X}{g:02X}{b:02X}")
}

fn color_text(text: &str, color: &str) -> String {
    // adds color to text 
    let h = color.trim_start_matches('#');
    let r = u8::from_str_radix(&h[0..2], 16).unwrap();
    let g = u8::from_str_radix(&h[2..4], 16).unwrap();
    let b = u8::from_str_radix(&h[4..6], 16).unwrap();

    // ANSI escape codes:
    // `\x1b` starts escape sequence
    // `[` starts control sequence introducer
    // `38` sets foreground color
    // `;2` indicates 24-bit color
    // `;r;g;b` 8-bit color channel values
    // `m` apply to following text
    // `\x1b[0m` resets all attributes
    format!("\x1b[38;2;{r};{g};{b}m{text}\x1b[0m")
}

fn color_background(text: &str, color: &str) -> String {
    // adds color to the background of text
    let h = color.trim_start_matches('#');
    let r = u8::from_str_radix(&h[0..2], 16).unwrap();
    let g = u8::from_str_radix(&h[2..4], 16).unwrap();
    let b = u8::from_str_radix(&h[4..6], 16).unwrap();

    // `48` sets background color
    format!("\x1b[48;2;{r};{g};{b}m{text}\x1b[0m")
}

fn color_to_state(color: &str) -> Option<(State, u8)> {
    match color {
        "4C545C" => {Some((State::Closed, 0))}
        "D8E0E8" => {Some((State::Flagged, 0))}
        "7B7B7B" => {Some((State::Mine, 0))}
        "384048" => {Some((State::Open, 0))}
        "7CC7FF" => {Some((State::Open, 1))}
        "66C266" => {Some((State::Open, 2))}
        "FF7788" => {Some((State::Open, 3))}
        "EE88FF" => {Some((State::Open, 4))}
        "DDAA22" => {Some((State::Open, 5))}
        "66CCCC" => {Some((State::Open, 6))}
        "888888" => {Some((State::Open, 7))}
        "D0D8E0" => {Some((State::Open, 8))}
        _ => {None}
    }
}

fn state_to_color(state: State, value: Option<u8>) -> String {
    match state {
        State::Closed => {"DCDCDC"}
        State::Flagged => {"F75656"}
        State::Mine => {"FF3333"}
        State::Open => {
            match value.unwrap_or(0) {
                1 => {"7CC7FF"}
                2 => {"66C266"}
                3 => {"FF7788"}
                4 => {"EE88FF"}
                5 => {"DDAA22"}
                6 => {"66CCCC"}
                7 => {"888888"}
                8 => {"D0D8E0"}
                _ => {"384048"}
            }
        }
    }.to_string()
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum State {
    Closed,
    Open,
    Flagged,
    Mine,
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum BoardState {
    Unsolved, // currently solving
    Solved, // board done
    Failed, // hit a mine
}


struct Board {
    corners: [u32; 4], // top left, bottom right, in screen px
    inner_board_corner: [u32; 2], // top left corner of inner board (where cells are)
    cell_size: u32, // width/height of cell square
    grid_size: [u32; 2], // how many cells are in the grid [w, h]
    mines: u32,
    mines_left: u32,
    cells: HashMap<(u32, u32), Cell>, // holds all cells and their data
    rag: RustAutoGui,
    monitor: Monitor,
    steps: u32, // current steps/actions taken
    state: BoardState, // solved, in progress, or failed
    draw: bool, // whether to draw the board/status messages
}

impl Board {
    fn initialize_board(&mut self) {
        for x in 1..self.grid_size[0]+1 {
            for y in 1..self.grid_size[1]+1 {
                self.cells.insert(
                    (x, y),
                    Cell {
                        x,
                        y,
                        state: State::Closed,
                        value: 0,
                        solved: false,
                    }
                );
            }
        }
    }

    fn get_cell_position(&self, x: u32, y: u32) -> Option<[u32; 2]> {
        if x > self.grid_size[0] || y > self.grid_size[1] {
            return None;
        }

        let center = self.cell_size / 2;
        let pos_x = self.inner_board_corner[0] + ((x - 1) * self.cell_size) + center;
        let pos_y = self.inner_board_corner[1] + ((y - 1) * self.cell_size) + center;

        Some([pos_x, pos_y])
    }

    fn get_cell_position_board(&self, x: u32, y: u32) -> Option<[u32; 2]> {
        // coordinates from the top left of the board
        if x > self.grid_size[0] || y > self.grid_size[1] {
            return None;
        }

        let center = self.cell_size / 2;
        let pos_x = (self.inner_board_corner[0] - self.corners[0]) + ((x - 1) * self.cell_size) + center;
        let pos_y = (self.inner_board_corner[1] - self.corners[1]) + ((y - 1) * self.cell_size) + center;

        Some([pos_x, pos_y])
    }

    fn get_cell_data(&self, x: u32, y: u32) -> Option<(State, u8)> {
        if x > self.grid_size[0] || y > self.grid_size[1] {
            return None;
        }
        let cell = self.cells.get(&(x, y))?;
        let state = cell.state;
        let value = cell.value;
        Some((state, value))
    }

    fn get_cell_state(&self, x: u32, y: u32) -> Option<State> {
        if x > self.grid_size[0] || y > self.grid_size[1] {
            return None;
        }
        let cell = self.cells.get(&(x, y))?;
        let state = cell.state;
        Some(state)
    }

    fn get_cell_value(&self, x: u32, y: u32) -> Option<u8> {
        if x > self.grid_size[0] || y > self.grid_size[1] {
            return None;
        }
        let cell = self.cells.get(&(x, y))?;
        let value = cell.value;
        Some(value)
    }

    fn get_cell_solved(&self, x: u32, y: u32) -> Option<bool> {
        if x > self.grid_size[0] || y > self.grid_size[1] {
            return None;
        }
        let cell = self.cells.get(&(x, y))?;
        let solved = cell.solved;
        Some(solved)
    }

    fn set_cell(&mut self, x: u32, y: u32, state: State, value: u8) {
        let cell = self.cells.get_mut(&(x, y));
        if let Some(cell) = cell {
            cell.state = state;
            cell.value = value;
        }
    }

    fn set_cell_state(&mut self, x: u32, y: u32, state: State) {
        let cell = self.cells.get_mut(&(x, y));
        if let Some(cell) = cell {
            cell.state = state;
        }
    }

    fn set_cell_value(&mut self, x: u32, y: u32, value: u8) {
        let cell = self.cells.get_mut(&(x, y));
        if let Some(cell) = cell {
            cell.value = value;
        }
    }

    fn set_cell_solved(&mut self, x: u32, y: u32, solved: bool) {
        let cell = self.cells.get_mut(&(x, y));
        if let Some(cell) = cell {
            cell.solved = solved;
        }
    }

    fn open_cell(&self, x: u32, y: u32) {
        let position = self.get_cell_position(x, y);
        if let Some(position) = position {
            let res = self.rag.move_mouse_to_pos(position[0], position[1], 0.0);
            match res {
                Ok(value) => {
                    self.rag.left_click();
                    wait(50);
                }
                Err(error) => {
                    println!("move mouse error: {error}");
                    exit(0);
                }
            }

        }
    }

    fn flag_cell(&mut self, x: u32, y: u32) {
        let position = self.get_cell_position(x, y);
        if let Some(position) = position {
            self.set_cell_state(x, y, State::Flagged);
            self.set_cell_solved(x, y, true);
            self.mines_left -= 1;
            self.rag.move_mouse_to_pos(position[0], position[1], 0.0);
            self.rag.right_click();
            wait(50);
        }
    }

    fn get_surrounding_cells(&self, x: u32, y: u32) -> Vec<&Cell> {
        // returns references of all valid surrounding cells
        let mut cells = Vec::new();
        if x > self.grid_size[0] || y > self.grid_size[1] {
            return cells;
        }

        let spots = [
            (x-1, y-1), (x, y-1), (x+1, y-1),
            (x-1, y  ),           (x+1, y  ),
            (x-1, y+1), (x, y+1), (x+1, y+1),
        ];

        if let Some(cell) = self.cells.get(&(x, y)) {
            for pos in spots {
                if let Some(surrounding_cell) = self.cells.get(&pos) {
                    cells.push(surrounding_cell);
                }
            }
        }

        cells
    }

    fn display_board(&self, status: &str) {
        if !self.draw {
            return;
        }
        // prints out the current minesweeper board state
        let w = self.grid_size[0];
        let h = self.grid_size[1];

        print!("\x1B[{}A", 3+h);
        Write::flush(&mut io::stdout()).unwrap();

        let mut board_text = String::new();

        let mines = format!("{:03}", self.mines_left);
        let spaces = " ".repeat(((w * 2 - 1 - 9) / 2) as usize); // spaces in between

        let face = match self.state {
            BoardState::Unsolved => {color_text(">_<", "#f3ff82")}
            BoardState::Solved => {color_text(">w<", "#7df084")}
            BoardState::Failed => {color_text("o_O", "#ff6e6e")}
        };
        let steps = format!("{:03}", self.steps);
        let top = mines + &spaces + &face + &spaces + &steps + "\n";
        board_text.push_str(&top);

        for y in 1..h+1 {
            let mut row = String::new();
            for x in 1..w+1 {
                let state = self.get_cell_state(x, y).unwrap_or(State::Mine);
                let value = self.get_cell_value(x, y).unwrap_or(0);
                let solved = self.get_cell_solved(x, y).unwrap_or(false);
                let color = &state_to_color(state, Some(value));

                let cell = match state {
                    State::Closed => {"~"}
                    State::Mine => {"X"}
                    State::Flagged => {"!"}
                    State::Open => {
                        if value > 0 && value < 9 {
                            &value.to_string()
                        } else {
                            " "
                        }
                    }
                };
                let text = if !solved {
                    // &color_text(cell, color)
                    &color_background(&color_text(&(cell.to_string() + " "), color), "#0d0e14")
                } else {
                    &color_background(&color_text(&(cell.to_string() + " "), color), "#1e1f29")
                };
                row = row + text;
            }

            row += "\n";
            board_text.push_str(&row);
        }

        // print!("\x1b[?1049h"); // switch to alternate screen
        // wait(1000);
        println!("{board_text}");
        println!("{status:<60}");
        // wait(1000);
        // print!("\x1b[?1049l");
    }
}

struct Cell {
    x: u32, // cell position in grid (starting at 0)
    y: u32,
    state: State, // closed/open/flagged/mine
    value: u8, // value/number of the cell if state=open
    solved: bool, // fully solved, skip checking this cell
}

impl Cell {

}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // seperate thread to listen for "Q" to quit immediately
    thread::spawn(|| {
        if let Err(error) = listen(move |event| {
            if let EventType::KeyPress(Key::KeyQ) = event.event_type {
                println!("Exiting...");
                exit(0);
            }
        }) {
            println!("Error: {error:?}")
        }
    });

    for i in (1..=2).rev() {
        print!("\rstarting in {i}...");
        Write::flush(&mut io::stdout()).unwrap();
        wait(1000);
    }
    println!("\r                 ");

    let start = Instant::now();
    let mut rag = RustAutoGui::new(false)?;
    let mut rng = rand::rng();

    let monitors = Monitor::all()?;
    dir::create_all("screenshots", true)?;
    let monitor = monitors
        .into_iter()
        .find(|m| m.is_primary().unwrap_or(false))
        .expect("No primary monitor found");

    let mut board = Board {
        // corners: [980, 251, 1400, 774],
        corners: [1173, 308, 2433, 1111],
        // inner_board_corner: [1010, 386],
        inner_board_corner: [1203, 443],
        cell_size: 40,
        grid_size: [30, 16],
        mines: 50,
        mines_left: 50,
        cells: HashMap::new(),
        rag,
        monitor,
        steps: 0,
        state: BoardState::Unsolved,
        draw: true,
    };

    board.initialize_board();

    let capture_width = board.corners[2] - board.corners[0];
    let capture_height = board.corners[3] - board.corners[1];

    // for looping through cells easier
    let x_range = 1..board.grid_size[0] + 1;
    let y_range = 1..board.grid_size[1] + 1;

    // print!("\x1B[{}B", 3+board.grid_size[1]);
    print!("{}", String::from("\n").repeat((3+board.grid_size[1]) as usize));
    board.display_board("Setting up");

    // ################ MAIN LOGIC LOOP ################
    let mut step_limit = 500;
    let wait_time = 500;
    let mut is_new = true;
    let mut should_update = true;
    let mut stuck_tries = 0; // loops before clicking random cell
    wait(wait_time);
    'main: loop {
        board.steps += 1;
        if board.steps > step_limit {
            break;
        }

        if should_update {
            should_update = false;
            board.display_board("Capturing screen");
            wait(wait_time);
            // ######## get current game state
            let image = board.monitor.capture_region(
                board.corners[0],
                board.corners[1],
                capture_width,
                capture_height
            )?;

            // println!(
            //     "> captured region {}x{} at ({}, {})",
            //     image.width(),
            //     image.height(),
            //     board.corners[0],
            //     board.corners[1],
            // );

            image.save(format!(
                "screenshots/monitor-{}-region.png",
                normalized(board.monitor.name()?) 
            ))?;

            board.display_board("Updating board");
            wait(wait_time);
            // ######## update board (if needed)
            for x in x_range.clone() {
                for y in y_range.clone() {
                    if let Some(solved) = board.get_cell_solved(x, y) && solved {
                        // cell already solved; continue
                        continue;
                    }

                    if let Some(state) = board.get_cell_state(x, y) && state != State::Closed {
                        // cell already opened; we don't need to update it
                        continue;
                    }

                    if let Some(pos) = board.get_cell_position_board(x, y) {
                        let pixel = image.get_pixel(pos[0], pos[1]);
                        let color = rgb_to_hex(pixel[0], pixel[1], pixel[2]);
                        if &color == "66DD66" {
                            board.display_board(&format!("No guess mode: Clicking first cell ({x}, {y})"));
                            wait(wait_time);
                            board.open_cell(x, y);
                            continue 'main;
                        }
                        let data = color_to_state(&color);
                        if let Some((state, value)) = data {
                            if is_new && state != State::Closed {
                                is_new = false;
                            }

                            if state == State::Open && value == 0 {
                                board.set_cell_solved(x, y, true);
                            }

                            if state == State::Flagged {
                                board.set_cell_solved(x, y, true);
                                board.mines_left -= 1;
                            }
                            board.set_cell(x, y, state, value);
                        } else {
                            println!("unknown color! cell ({}, {}) color={}", x, y, color_text(&color, &color));
                            exit(0)
                        }
                    }
                }
            }
        }

        // ######## check for any mines
        for x in x_range.clone() {
            for y in y_range.clone() {
                if let Some(state) = board.get_cell_state(x, y) && state == State::Mine {
                    board.state = BoardState::Failed;
                    board.display_board("Board failed! D:");
                    wait(wait_time);
                    break 'main;
                }
            }
        }

        // ######## if board is new, click random cell
        if is_new {
            board.display_board("Picking first cell");
            wait(wait_time);
            let rand_x = rng.random_range(1..=board.grid_size[0]);
            let rand_y = rng.random_range(1..=board.grid_size[1]);
            board.display_board(&format!("Clicking cell ({rand_x}, {rand_y})"));
            wait(wait_time);
            board.open_cell(rand_x, rand_y);
            should_update = true;
            continue 'main;
        }

        // ######## check for guaranteed mines or easy safe opens
        board.display_board("Checking surrounding tiles");
        wait(wait_time);
        for x in x_range.clone() {
            for y in y_range.clone() {
                if let Some(solved) = board.get_cell_solved(x, y) && solved {
                    // cell already solved; continue
                    continue;
                }

                if let Some((state, value)) = board.get_cell_data(x, y)
                    && state == State::Open && value != 0 {
                    // this is a cell with a number != 0; check how many surrounding cells are unopened
                    let mut closed = Vec::new();
                    let mut flagged = Vec::new();
                    let surrounding_cells = board.get_surrounding_cells(x, y);

                    for cell in surrounding_cells {
                        if cell.state == State::Closed {
                            closed.push(cell);
                        }
                        if cell.state == State::Flagged {
                            flagged.push(cell);
                        }
                    }

                    if value == flagged.len() as u8 {
                        // all surrounding mines are flagged; open closed cells (if any) or mark as solved
                        if !closed.is_empty() {
                            let cell = closed[0];
                            board.display_board(&format!("Opening cell ({},{}) from ({},{})", cell.x, cell.y, x, y));
                            wait(wait_time);
                            board.open_cell(cell.x, cell.y);
                            should_update = true;
                            continue 'main;
                        }
                        board.display_board(&format!("Marking cell ({x},{y}) solved: All surrounding mines flagged"));
                        wait(wait_time);
                        board.set_cell_solved(x, y, true);
                        continue 'main;
                    }
                    
                    if value == (flagged.len() + closed.len()) as u8 && !closed.is_empty() {
                        // surrounding unopened tiles match mine count; flag cells
                        let cell = closed[0];
                        board.display_board(&format!("Flagging cell ({},{}) from ({},{})", cell.x, cell.y, x, y));
                        wait(wait_time);
                        board.flag_cell(cell.x, cell.y);
                        continue 'main;
                    }
                }
            }
        }

        // ######## check if whole board is solved
        let mut board_solved = true;
        'solve_check: for x in x_range.clone() {
            for y in y_range.clone() {
                if let Some(solved) = board.get_cell_solved(x, y) && !solved {
                    board_solved = false;
                    break 'solve_check;
                }
            }
        }
        if board_solved {
            board.state = BoardState::Solved;
            board.display_board("Board solved! :D");
            wait(wait_time);
            break 'main;
        }

        if stuck_tries >= 3 {
            // ######## nothing else to do, click random cell
            board.display_board("I'm stuck! Picking random cell");
            wait(wait_time);
            'random: loop {
                let rand_x = rng.random_range(1..=board.grid_size[0]);
                let rand_y = rng.random_range(1..=board.grid_size[1]);
                if let Some(state) = board.get_cell_state(rand_x, rand_y) && state == State::Closed {
                    board.display_board(&format!("Clicking cell ({rand_x}, {rand_y})"));
                    wait(wait_time);
                    board.open_cell(rand_x, rand_y);
                    should_update = true;
                    break 'random;
                }
            }
        }

        stuck_tries += 1;
        should_update = true;
    }

    // for cell in board.cells.values() {
    //     println!("cell ({}, {}) is {:?}, value: {}", cell.x, cell.y, cell.state, cell.value);
    // }

    // board.set_cell(2, 2, State::Open, 2);
    // board.set_cell(3, 2, State::Open, 1);
    // board.set_cell(4, 2, State::Open, 3);
    // board.set_cell(6, 2, State::Open, 3);
    // board.set_cell(7, 2, State::Open, 2);
    // board.set_cell(8, 2, State::Open, 3);
    // board.set_cell(2, 3, State::Open, 1);
    // board.set_cell(4, 3, State::Open, 1);
    // board.set_cell(5, 3, State::Open, 1);
    // board.set_cell(6, 3, State::Open, 1);
    // board.set_cell(8, 3, State::Open, 1);
    // board.set_cell(1, 4, State::Open, 1);
    // board.set_cell(2, 4, State::Open, 1);
    // board.set_cell(8, 4, State::Open, 1);
    // board.set_cell(7, 5, State::Open, 1);
    // board.set_cell(8, 5, State::Open, 2);
    // board.set_cell(1, 6, State::Open, 2);
    // board.set_cell(2, 6, State::Open, 1);
    // board.set_cell(6, 6, State::Open, 1);
    // board.set_cell(7, 6, State::Open, 2);
    // board.set_cell(2, 7, State::Open, 1);
    // board.set_cell(4, 7, State::Open, 1);
    // board.set_cell(5, 7, State::Open, 1);
    // board.set_cell(6, 7, State::Open, 2);
    // board.set_cell(1, 8, State::Open, 2);
    // board.set_cell(2, 8, State::Open, 1);
    // board.set_cell(4, 8, State::Open, 1);
    // board.set_cell(4, 9, State::Open, 1);

    // board.set_cell(3, 3, State::Open, 0);
    // board.set_cell(7, 3, State::Open, 0);
    // board.set_cell(3, 4, State::Open, 0);
    // board.set_cell(4, 4, State::Open, 0);
    // board.set_cell(5, 4, State::Open, 0);
    // board.set_cell(6, 4, State::Open, 0);
    // board.set_cell(7, 4, State::Open, 0);
    // board.set_cell(1, 5, State::Open, 0);
    // board.set_cell(2, 5, State::Open, 0);
    // board.set_cell(3, 5, State::Open, 0);
    // board.set_cell(4, 5, State::Open, 0);
    // board.set_cell(5, 5, State::Open, 0);
    // board.set_cell(6, 5, State::Open, 0);
    // board.set_cell(3, 6, State::Open, 0);
    // board.set_cell(4, 6, State::Open, 0);
    // board.set_cell(5, 6, State::Open, 0);
    // board.set_cell(3, 7, State::Open, 0);
    // board.set_cell(3, 8, State::Open, 0);
    // board.set_cell(1, 9, State::Open, 0);
    // board.set_cell(2, 9, State::Open, 0);
    // board.set_cell(3, 9, State::Open, 0);
    
    // board.set_cell(1, 7, State::Mine, 0);

    // board.open_cell(9, 9);
    // board.open_cell(8, 9);
    // board.open_cell(7, 9);
    // board.open_cell(6, 9);
    // board.open_cell(5, 9);
    // board.open_cell(4, 9);
    // board.open_cell(3, 9);
    // board.open_cell(2, 9);
    // board.open_cell(1, 9);

    // let mouse_pos = board.rag.get_mouse_position()?;
    // println!("mouse position: ({}, {})", mouse_pos.0, mouse_pos.1);

    println!("\nruntime: {:?}\nsteps: {}", start.elapsed(), board.steps);

    Ok(())
}