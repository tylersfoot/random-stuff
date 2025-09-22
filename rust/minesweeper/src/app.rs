use std::{collections::HashMap};

pub enum CurrentScreen {
    Menu,
    // Settings,
    Gameplay,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum CellState {
    Closed,
    Open,
    Flagged,
    Mine,
}

pub struct Cell {
    pub x: u32, // cell position in grid (starting at 0)
    pub y: u32,
    pub state: CellState, // closed/open/flagged/mine
    pub value: u8, // value/number of the cell if state=open
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum BoardState {
    InProgress, // currently playing
    Solved,     // board has been solved
    Failed,     // hit a mine
}

pub struct Board {
    pub grid_size: [u32; 2], // how many cells are in the grid [w, h]
    pub mines: u32,
    pub mines_left: u32,
    pub cells: HashMap<(u32, u32), Cell>, // holds all cells and their data
    pub state: BoardState, // solved, in progress, or failed
}

impl Board {
    pub fn new_empty() -> Board {
        Board {
            grid_size: [0, 0],
            mines: 0,
            mines_left: 0,
            cells: HashMap::new(),
            state: BoardState::InProgress,
        }
    }

    pub fn new_example() -> Board {
        let grid_size= [8, 9];
        let mut cells = HashMap::new();
        for x in 1..=grid_size[0] {
            for y in 1..=grid_size[1] {
                cells.insert(
                    (x, y),
                    Cell {
                        x,
                        y,
                        state: CellState::Closed,
                        value: 0,
                    }
                );
            }
        }
        
        let mut board = Board {
            grid_size,
            mines: 50,
            mines_left: 40,
            cells,
            state: BoardState::InProgress,
        };

        // open
        board.set_cell(2, 2, CellState::Open, 2);
        board.set_cell(3, 2, CellState::Open, 1);
        board.set_cell(4, 2, CellState::Open, 3);
        board.set_cell(6, 2, CellState::Open, 3);
        board.set_cell(7, 2, CellState::Open, 2);
        board.set_cell(8, 2, CellState::Open, 3);
        board.set_cell(2, 3, CellState::Open, 1);
        board.set_cell(4, 3, CellState::Open, 1);
        board.set_cell(5, 3, CellState::Open, 1);
        board.set_cell(6, 3, CellState::Open, 1);
        board.set_cell(8, 3, CellState::Open, 1);
        board.set_cell(1, 4, CellState::Open, 1);
        board.set_cell(2, 4, CellState::Open, 1);
        board.set_cell(8, 4, CellState::Open, 1);
        board.set_cell(7, 5, CellState::Open, 1);
        board.set_cell(8, 5, CellState::Open, 2);
        board.set_cell(1, 6, CellState::Open, 2);
        board.set_cell(2, 6, CellState::Open, 1);
        board.set_cell(6, 6, CellState::Open, 1);
        board.set_cell(7, 6, CellState::Open, 2);
        board.set_cell(2, 7, CellState::Open, 1);
        board.set_cell(4, 7, CellState::Open, 1);
        board.set_cell(5, 7, CellState::Open, 1);
        board.set_cell(6, 7, CellState::Open, 2);
        board.set_cell(1, 8, CellState::Open, 2);
        board.set_cell(2, 8, CellState::Open, 1);
        board.set_cell(4, 8, CellState::Open, 1);
        board.set_cell(4, 9, CellState::Open, 1);

        // blanks
        board.set_cell(3, 3, CellState::Open, 0);
        board.set_cell(7, 3, CellState::Open, 0);
        board.set_cell(3, 4, CellState::Open, 0);
        board.set_cell(4, 4, CellState::Open, 0);
        board.set_cell(5, 4, CellState::Open, 0);
        board.set_cell(6, 4, CellState::Open, 0);
        board.set_cell(7, 4, CellState::Open, 0);
        board.set_cell(1, 5, CellState::Open, 0);
        board.set_cell(2, 5, CellState::Open, 0);
        board.set_cell(3, 5, CellState::Open, 0);
        board.set_cell(4, 5, CellState::Open, 0);
        board.set_cell(5, 5, CellState::Open, 0);
        board.set_cell(6, 5, CellState::Open, 0);
        board.set_cell(3, 6, CellState::Open, 0);
        board.set_cell(4, 6, CellState::Open, 0);
        board.set_cell(5, 6, CellState::Open, 0);
        board.set_cell(3, 7, CellState::Open, 0);
        board.set_cell(3, 8, CellState::Open, 0);
        board.set_cell(1, 9, CellState::Open, 0);
        board.set_cell(2, 9, CellState::Open, 0);
        board.set_cell(3, 9, CellState::Open, 0);
        
        // mine
        board.set_cell(1, 7,CellState::Mine, 0);

        board
    }

    pub fn get_cell_data(&self, x: u32, y: u32) -> Option<(CellState, u8)> {
        if x > self.grid_size[0] || y > self.grid_size[1] {
            return None;
        }
        let cell = self.cells.get(&(x, y))?;
        let state = cell.state;
        let value = cell.value;
        Some((state, value))
    }

    pub fn get_cell_state(&self, x: u32, y: u32) -> Option<CellState> {
        if x > self.grid_size[0] || y > self.grid_size[1] {
            return None;
        }
        let cell = self.cells.get(&(x, y))?;
        let state = cell.state;
        Some(state)
    }

    pub fn get_cell_value(&self, x: u32, y: u32) -> Option<u8> {
        if x > self.grid_size[0] || y > self.grid_size[1] {
            return None;
        }
        let cell = self.cells.get(&(x, y))?;
        let value = cell.value;
        Some(value)
    }

    pub fn set_cell(&mut self, x: u32, y: u32, state: CellState, value: u8) {
        let cell = self.cells.get_mut(&(x, y));
        if let Some(cell) = cell {
            cell.state = state;
            cell.value = value;
        }
    }

    pub fn set_cell_state(&mut self, x: u32, y: u32, state: CellState) {
        let cell = self.cells.get_mut(&(x, y));
        if let Some(cell) = cell {
            cell.state = state;
        }
    }

    pub fn set_cell_value(&mut self, x: u32, y: u32, value: u8) {
        let cell = self.cells.get_mut(&(x, y));
        if let Some(cell) = cell {
            cell.value = value;
        }
    }
}

pub struct App {
    pub current_screen: CurrentScreen, // which screen to display
    pub board: Board, // the minesweeper board
}

impl App {
    pub fn new() -> App {
        App {
            current_screen: CurrentScreen::Menu,
            board: Board::new_example(),
        }
    }
}