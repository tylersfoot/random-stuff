use ratatui::{
    layout::{Constraint, Direction, Flex, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame
};

use crate::app::{App, Board, BoardState, CellState, CurrentScreen};

fn normalized(filename: String) -> String {
    filename.replace(['|', '\\', ':', '/'], "")
}

fn rgb_to_hex(r: u8, g: u8, b: u8) -> String {
    format!("{r:02X}{g:02X}{b:02X}")
}

fn color_text(text: &str, color: &str) -> String {
    String::from(text)
    // // adds color to text 
    // let h = color.trim_start_matches('#');
    // let r = u8::from_str_radix(&h[0..2], 16).unwrap();
    // let g = u8::from_str_radix(&h[2..4], 16).unwrap();
    // let b = u8::from_str_radix(&h[4..6], 16).unwrap();

    // // ANSI escape codes:
    // // `\x1b` starts escape sequence
    // // `[` starts control sequence introducer
    // // `38` sets foreground color
    // // `;2` indicates 24-bit color
    // // `;r;g;b` 8-bit color channel values
    // // `m` apply to following text
    // // `\x1b[0m` resets all attributes
    // format!("\x1b[38;2;{r};{g};{b}m{text}\x1b[0m")
}

fn color_background(text: &str, color: &str) -> String {
    String::from(text)
    // // adds color to the background of text
    // let h = color.trim_start_matches('#');
    // let r = u8::from_str_radix(&h[0..2], 16).unwrap();
    // let g = u8::from_str_radix(&h[2..4], 16).unwrap();
    // let b = u8::from_str_radix(&h[4..6], 16).unwrap();

    // // `48` sets background color
    // format!("\x1b[48;2;{r};{g};{b}m{text}\x1b[0m")
}

fn state_to_color(state: CellState, value: Option<u8>) -> String {
    match state {
        CellState::Closed => {"DCDCDC"}
        CellState::Flagged => {"F75656"}
        CellState::Mine => {"FF3333"}
        CellState::Open => {
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

pub fn ui(frame: &mut Frame, app: &App) {
    // create the layout sections
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(3),
        ])
        .split(frame.area());

    let title_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default());

    let title = Paragraph::new(Text::styled(
        "Create New Json",
        Style::default().fg(Color::Green),
    ))
    .block(title_block);

    frame.render_widget(title, chunks[0]);

    let current_navigation_text = vec![
        // The first half of the text
        match app.current_screen {
            CurrentScreen::Menu => Span::styled("Menu", Style::default().fg(Color::Green)),
            CurrentScreen::Gameplay => Span::styled("Gameplay", Style::default().fg(Color::LightRed)),
        }
        .to_owned(),
        // A white divider bar to separate the two sections
        Span::styled(" | ", Style::default().fg(Color::White)),
        // The final section of the text, with hints on what the user is editing
        {
            Span::styled("Not Editing Anything", Style::default().fg(Color::DarkGray))
        },
    ];

    let mode_footer = Paragraph::new(Line::from(current_navigation_text))
        .block(Block::default().borders(Borders::ALL));

    let current_keys_hint = {
        match app.current_screen {
            CurrentScreen::Menu => Span::styled(
                "menu: (q) to quit / (e) to enter gameplay",
                Style::default().fg(Color::Red),
            ),
            CurrentScreen::Gameplay => Span::styled(
                "gameplay: (q) to go back to menu",
                Style::default().fg(Color::Red),
            ),
        }
    };

    let key_notes_footer =
        Paragraph::new(Line::from(current_keys_hint)).block(Block::default().borders(Borders::ALL));

    let footer_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[2]);

    frame.render_widget(mode_footer, footer_chunks[0]);
    frame.render_widget(key_notes_footer, footer_chunks[1]);

    if let CurrentScreen::Gameplay = app.current_screen {
        // clear the entire screen and anything already drawn
        frame.render_widget(Clear, frame.area());

        let screen = frame.area();

        let w = app.board.grid_size[0];
        let h = app.board.grid_size[1];
        let border_padding = 2; // to allow space for border to draw
        let w_px = (w * 2 + 1 + border_padding) as u16;
        let h_px = (h + 1 + border_padding) as u16;
        let x = (screen.width - w_px) / 2;
        let y = (screen.height - h_px) / 2;

        let area = Rect::new(x, y, w_px, h_px);


        // display the current minesweeper board state
        let mut board_text = String::new();

        let mines = format!("{:03}", app.board.mines_left);
        let spaces = " ".repeat(((w * 2 - 1 - 9) / 2) as usize); // spaces in between

        // let r = u8::from_str_radix(&h[0..2], 16).unwrap();
        // let g = u8::from_str_radix(&h[2..4], 16).unwrap();
        // let b = u8::from_str_radix(&h[4..6], 16).unwrap();

        let (r, g, b) = (243, 255, 130);
        let color = Color::Rgb(r, g, b);

        let p1 = Paragraph::new("hi");
        let p2 = Paragraph::new("yo");
        let p3 = p1 + p2;

        let aa = format!("\x1b[38;2;{r};{g};{b}m>_<\x1b[0m");


        let face = match app.board.state {
            // BoardState::InProgress => {color_text(">_<", "#f3ff82")}
            BoardState::InProgress => {aa}
            BoardState::Solved => {color_text(">w<", "#7df084")}
            BoardState::Failed => {color_text("o_O", "#ff6e6e")}
        };
        let steps = format!("{:03}", 30);
        let top = String::from(" ") + &mines + &spaces + &face + &spaces + &steps + "\n";
        board_text.push_str(&top);

        for y in 1..h+1 {
            let mut row = String::new();
            row += " ";
            for x in 1..w+1 {
                let state = app.board.get_cell_state(x, y).unwrap_or(CellState::Mine);
                let value = app.board.get_cell_value(x, y).unwrap_or(0);
                let color = &state_to_color(state, Some(value));

                let cell = match state {
                    CellState::Closed => {"~"}
                    CellState::Mine => {"X"}
                    CellState::Flagged => {"!"}
                    CellState::Open => {
                        if value > 0 && value < 9 {
                            &value.to_string()
                        } else {
                            " "
                        }
                    }
                };
                let text = &color_background(&color_text(&(cell.to_string() + " "), color), "#0d0e14");
                row = row + text;
            }

            row += "\n";
            board_text.push_str(&row);
        }

        let paragraph = Paragraph::new(board_text).block(Block::bordered());

        frame.render_widget(paragraph, area);

        // frame.render_widget(Block::bordered(), area);
        frame.render_widget(Block::bordered(), screen);


        // let col_constraints = (0..w).map(|_| Constraint::Length(2));
        // let row_constraints = (0..h).map(|_| Constraint::Length(1));
        // let horizontal = Layout::horizontal(col_constraints).spacing(0);
        // let vertical = Layout::vertical(row_constraints).spacing(0);

        // let rows = vertical.split(area);
        // let cells = rows.iter().flat_map(|&row| horizontal.split(row).to_vec());

        // for cell in cells {
        //     let p = Paragraph::new("X")
        //             .block(Block::new());
        //     frame.render_widget(p, cell);
        // }

        // let mut text = app.board.display_board();
        // text.truncate(500);
        // let text = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.";
        // let paragraph = Paragraph::new(text.dark_gray()).wrap(Wrap { trim: true });
        // let paragraph = Paragraph::new(text);


        // let gameplay_block = Block::new()
        //     .borders(Borders::ALL);
        // frame.render_widget(paragraph.clone().block(gameplay_block), area);



        // let exit_text = Text::styled(
        //     "This is the gameplay screen!",
        //     Style::default().fg(Color::Red),
        // );

        // let exit_paragraph = Paragraph::new(exit_text)
        //     .block(gameplay_block);

        // frame.render_widget(exit_paragraph, area);
    }
}

fn center(area: Rect, horizontal: Constraint, vertical: Constraint) -> Rect {
    let [area] = Layout::horizontal([horizontal])
        .flex(Flex::Center)
        .areas(area);
    let [area] = Layout::vertical([vertical]).flex(Flex::Center).areas(area);
    area
}

/// helper function to create a centered rect using up certain percentage of the available rect `r`
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    // Cut the given rectangle into three vertical pieces
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    // Then cut the middle vertical piece into three width-wise pieces
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1] // Return the middle chunk
}