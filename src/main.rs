use ratatui::{
    crossterm::{
        event::{
            self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyEventKind,
            MouseButton, MouseEvent, MouseEventKind,
        },
        ExecutableCommand,
    },
    layout::Flex,
    prelude::*,
    widgets::Block,
};
use std::collections::BTreeMap;

#[derive(Clone, Copy)]
enum PieceType {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

impl PieceType {
    fn render(&self) -> &'static str {
        match self {
            Self::Pawn => "♟",
            Self::Knight => "♞",
            Self::Bishop => "♝",
            Self::Rook => "♜",
            Self::Queen => "♛",
            Self::King => "♚",
        }
    }
}

#[derive(Clone, Copy)]
enum PieceColor {
    White,
    Black,
}

impl PieceColor {
    fn render(&self) -> Color {
        match self {
            Self::White => Color::White,
            Self::Black => Color::Black,
        }
    }
}

#[derive(Clone, Copy)]
struct Piece {
    color: PieceColor,
    piece: PieceType,
}

impl Piece {
    fn render(&self) -> Span<'static> {
        Span::raw(self.piece.render()).fg(self.color.render())
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
struct Position {
    rank: usize,
    file: usize,
}

impl Position {
    fn square(&self) -> Line<'static> {
        let bg = if (self.rank + self.file) % 2 == 0 {
            Color::DarkGray
        } else {
            Color::Gray
        };
        Line::default().bg(bg)
    }
}

struct Board(BTreeMap<Position, Piece>);

impl Board {
    fn new() -> Self {
        const PIECE_TYPES: [PieceType; 8] = [
            PieceType::Rook,
            PieceType::Knight,
            PieceType::Bishop,
            PieceType::Queen,
            PieceType::King,
            PieceType::Bishop,
            PieceType::Knight,
            PieceType::Rook,
        ];
        let color = PieceColor::Black;
        let black_pieces = PIECE_TYPES
            .into_iter()
            .enumerate()
            .map(|(file, piece)| (Position { rank: 7, file }, Piece { color, piece }));
        let piece = PieceType::Pawn;
        let black_pawns = (0..8).map(|file| (Position { rank: 6, file }, Piece { color, piece }));
        let color = PieceColor::White;
        let white_pawns = (0..8).map(|file| (Position { rank: 1, file }, Piece { color, piece }));
        let white_pieces = PIECE_TYPES
            .into_iter()
            .enumerate()
            .map(|(file, piece)| (Position { rank: 0, file }, Piece { color, piece }));
        Self(
            black_pieces
                .chain(black_pawns)
                .chain(white_pawns)
                .chain(white_pieces)
                .collect(),
        )
    }

    fn get(&self, position: &Position) -> Option<&Piece> {
        self.0.get(position)
    }
}

struct Game {
    board: Board,
}

impl Game {
    fn new() -> Self {
        let board = Board::new();
        Self { board }
    }
}

struct Tui {
    game: Game,
    click_targets: Vec<(Rect, Position)>,
    selected_tile: Option<Position>,
}

impl Tui {
    fn new() -> Self {
        let game = Game::new();
        let click_targets = Vec::new();
        Self {
            game,
            click_targets,
            selected_tile: None,
        }
    }

    fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> std::io::Result<()> {
        // TODO: loop only if game is in progress
        loop {
            terminal.draw(|frame| frame.render_widget(&mut *self, frame.area()))?;
            let event = event::read()?;
            if self.handle(event) {
                break;
            }
        }
        Ok(())
    }

    /// returns whether to exit
    fn handle(&mut self, event: Event) -> bool {
        match event {
            Event::Key(KeyEvent {
                code: KeyCode::Esc | KeyCode::Char('q'),
                kind: KeyEventKind::Press,
                modifiers: _,
                state: _,
            }) => true,
            Event::Mouse(MouseEvent {
                kind: MouseEventKind::Down(MouseButton::Left),
                column,
                row,
                modifiers: _,
            }) => {
                let click = ratatui::layout::Position { x: column, y: row };
                for &(rect, position) in &self.click_targets {
                    if rect.contains(click) {
                        self.selected_tile =
                            self.game.board.get(&position).is_some().then_some(position);
                        break;
                    }
                }
                false
            }
            _ => false,
        }
    }
}

impl Widget for &mut Tui {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let [area] = Layout::horizontal([8 * 2]).flex(Flex::Center).areas(area);
        let [area] = Layout::vertical([8]).flex(Flex::Center).areas(area);
        let ranks = Layout::vertical([Constraint::Fill(1); 8]).split(area);
        self.click_targets.clear();
        for (rank, rect) in ranks.iter().copied().rev().enumerate() {
            let files = Layout::horizontal([Constraint::Fill(1); 8]).split(rect);
            for (file, rect) in files.iter().copied().enumerate() {
                let position = Position { rank, file };
                let mut line = position.square();
                if let Some(piece) = self.game.board.get(&position) {
                    line.push_span(piece.render())
                } else {
                    line.push_span(" ")
                }
                if self.selected_tile == Some(position) {
                    // TODO: use this for places to move to: ○
                    line.push_span(Span::raw("●").fg(Color::LightYellow))
                }
                line.render(rect, buf);
                self.click_targets.push((rect, position));
            }
        }
    }
}

fn main() -> Result<(), std::io::Error> {
    let mut tui = Tui::new();
    let mut terminal = ratatui::init();
    terminal
        .backend_mut()
        .writer_mut()
        .execute(EnableMouseCapture)?;
    if let Err(e) = tui.run(&mut terminal) {
        eprintln!("{e}");
    }
    terminal
        .backend_mut()
        .writer_mut()
        .execute(DisableMouseCapture)?;
    ratatui::restore();
    Ok(())
}
