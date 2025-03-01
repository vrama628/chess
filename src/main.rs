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
};
use std::collections::{BTreeMap, BTreeSet};

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

#[derive(Clone, Copy, PartialEq, Eq)]
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

    fn up(&self) -> Self {
        Self {
            rank: self.rank + 1,
            file: self.file,
        }
    }

    fn down(&self) -> Self {
        Self {
            rank: self.rank.wrapping_sub(1),
            file: self.file,
        }
    }

    fn left(&self) -> Self {
        Self {
            rank: self.rank,
            file: self.file.wrapping_sub(1),
        }
    }

    fn right(&self) -> Self {
        Self {
            rank: self.rank,
            file: self.file + 1,
        }
    }

    fn is_valid(&self) -> bool {
        self.rank < 8 && self.file < 8
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

    /// REQUIRES: there is a piece at `from`
    fn r#move(&mut self, from: Position, to: Position) {
        let piece = self.0.remove(&from).expect("Board::r#move precondition");
        self.0.insert(to, piece);
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

    /// REQUIRES: there is a piece at this position
    /// TODO: promotion, en passant, castling
    /// meh, for now, just auto-promote to queen
    fn moves(&self, position: &Position) -> BTreeSet<Position> {
        let Piece { color, piece } = self.board.get(position).expect("Game::moves precondition");
        let try_insert = |moves: &mut BTreeSet<Position>, p: Position| {
            if position.is_valid() && self.board.get(&p).is_none_or(|other| other.color != *color) {
                moves.insert(p);
            }
        };
        let saturate = |moves: &mut BTreeSet<Position>, f: &dyn Fn(Position) -> Position| {
            let mut moved = *position;
            while moved.is_valid() {
                moved = f(moved);
                if let Some(other) = self.board.get(&moved) {
                    if other.color != *color {
                        moves.insert(moved);
                    }
                    break;
                } else {
                    moves.insert(moved);
                }
            }
        };
        let mut moves = BTreeSet::new();
        match piece {
            PieceType::Pawn => match color {
                // TODO: promotion, en passant
                PieceColor::White => {
                    try_insert(&mut moves, position.up());
                    if position.rank == 1 {
                        try_insert(&mut moves, position.up().up());
                    }
                    if self
                        .board
                        .get(&position.up().right())
                        .is_some_and(|other| other.color == PieceColor::Black)
                    {
                        try_insert(&mut moves, position.up().right());
                    }
                    if self
                        .board
                        .get(&position.up().left())
                        .is_some_and(|other| other.color == PieceColor::Black)
                    {
                        try_insert(&mut moves, position.up().left());
                    }
                }
                PieceColor::Black => {
                    try_insert(&mut moves, position.down());
                    if position.rank == 6 {
                        try_insert(&mut moves, position.down().down());
                    }
                    if self
                        .board
                        .get(&position.down().right())
                        .is_some_and(|other| other.color == PieceColor::White)
                    {
                        try_insert(&mut moves, position.down().right());
                    }
                    if self
                        .board
                        .get(&position.down().left())
                        .is_some_and(|other| other.color == PieceColor::White)
                    {
                        try_insert(&mut moves, position.down().left());
                    }
                }
            },
            PieceType::Knight => {
                try_insert(&mut moves, position.up().up().left());
                try_insert(&mut moves, position.up().up().right());
                try_insert(&mut moves, position.left().left().up());
                try_insert(&mut moves, position.left().left().down());
                try_insert(&mut moves, position.down().down().left());
                try_insert(&mut moves, position.down().down().right());
                try_insert(&mut moves, position.right().right().up());
                try_insert(&mut moves, position.right().right().down());
            }
            PieceType::Bishop => {
                saturate(&mut moves, &|p| p.up().left());
                saturate(&mut moves, &|p| p.up().right());
                saturate(&mut moves, &|p| p.down().left());
                saturate(&mut moves, &|p| p.down().right());
            }
            PieceType::Rook => {
                saturate(&mut moves, &|p| p.up());
                saturate(&mut moves, &|p| p.left());
                saturate(&mut moves, &|p| p.down());
                saturate(&mut moves, &|p| p.right());
            }
            PieceType::Queen => {
                saturate(&mut moves, &|p| p.up());
                saturate(&mut moves, &|p| p.left());
                saturate(&mut moves, &|p| p.down());
                saturate(&mut moves, &|p| p.right());
                saturate(&mut moves, &|p| p.up().left());
                saturate(&mut moves, &|p| p.up().right());
                saturate(&mut moves, &|p| p.down().left());
                saturate(&mut moves, &|p| p.down().right());
            }
            PieceType::King => {
                // TODO: castling
                // TODO: avoid check
                try_insert(&mut moves, position.up());
                try_insert(&mut moves, position.up().right());
                try_insert(&mut moves, position.right());
                try_insert(&mut moves, position.down().right());
                try_insert(&mut moves, position.down());
                try_insert(&mut moves, position.down().left());
                try_insert(&mut moves, position.left());
                try_insert(&mut moves, position.up().left());
            }
        }
        moves
    }

    /// REQUIRES: there is a piece at `from`
    fn r#move(&mut self, from: Position, to: Position) {
        self.board.r#move(from, to);
    }
}

struct Tui {
    game: Game,
    click_targets: Vec<(Rect, Position)>,
    selected_tile: Option<(Position, BTreeSet<Position>)>,
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
                        match self.selected_tile {
                            None => self.set_selected_tile(position),
                            Some((from, ref moves)) => {
                                if moves.contains(&position) {
                                    self.game.r#move(from, position);
                                    self.selected_tile = None;
                                } else {
                                    self.set_selected_tile(position);
                                }
                            }
                        }
                        break;
                    }
                }
                false
            }
            _ => false,
        }
    }

    fn set_selected_tile(&mut self, position: Position) {
        self.selected_tile = self
            .game
            .board
            .get(&position)
            .is_some()
            .then(|| (position, self.game.moves(&position)));
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
                if self
                    .selected_tile
                    .as_ref()
                    .is_some_and(|(p, _)| *p == position)
                {
                    line.push_span(Span::raw("●").fg(Color::LightYellow))
                }
                if self
                    .selected_tile
                    .as_ref()
                    .is_some_and(|(_, moves)| moves.contains(&position))
                {
                    line.push_span(Span::raw("○").fg(Color::LightGreen))
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
