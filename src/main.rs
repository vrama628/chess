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
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Display,
    ops::Not,
    process::ExitCode,
};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
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

impl Not for PieceColor {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Self::White => Self::Black,
            Self::Black => Self::White,
        }
    }
}

impl Display for PieceColor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PieceColor::White => write!(f, "White"),
            PieceColor::Black => write!(f, "Black"),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
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

impl Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", (b'a' + self.file as u8) as char, self.rank + 1)
    }
}

#[derive(Clone)]
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
    fn r#move(&self, from: Position, to: Move) -> Self {
        // TODO: use im
        let mut new = self.clone();
        let piece = new.0.remove(&from).expect("Board::r#move precondition");
        new.0.insert(to, piece);
        new
    }

    fn promote(&self, from: Position, to: Move, piece_type: PieceType) -> Self {
        // TODO: use im
        let mut new = self.clone();
        let Piece { color, piece } = new.0.remove(&from).expect("Board::promote precondition");
        debug_assert_eq!(piece, PieceType::Pawn);
        let piece = Piece {
            color,
            piece: piece_type,
        };
        new.0.insert(to, piece);
        new
    }

    fn position_of(&self, piece: Piece) -> Option<Position> {
        self.0
            .iter()
            .find_map(|(&position, &p)| (p == piece).then_some(position))
    }
}

struct Game {
    turn: PieceColor,
    board: Board,
}

// TODO: en passant, castling, promotion
type Move = Position;

enum Outcome {
    Win(PieceColor),
    Draw,
}

impl Display for Outcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Outcome::Win(color) => writeln!(f, "{color} wins!"),
            Outcome::Draw => writeln!(f, "Draw!"),
        }
    }
}

impl Game {
    fn new() -> Self {
        let turn = PieceColor::White;
        let board = Board::new();
        Self { turn, board }
    }

    /// TODO: promotion, en passant, castling
    /// returns moves that can be made, but without filtering out moves that would put the king in check
    fn potential_moves(&self, color: PieceColor) -> BTreeMap<Position, BTreeSet<Move>> {
        self.board
            .0
            .iter()
            .filter(|&(_, &piece)| piece.color == color)
            .map(|(&from, &Piece { color, piece })| {
                let try_insert = |moves: &mut BTreeSet<Position>, to: Position| {
                    if to.is_valid() && self.board.get(&to).is_none_or(|other| other.color != color)
                    {
                        moves.insert(to);
                    }
                };
                let saturate = |moves: &mut BTreeSet<Position>,
                                f: &dyn Fn(Position) -> Position| {
                    let mut to = from;
                    while to.is_valid() {
                        to = f(to);
                        if let Some(other) = self.board.get(&to) {
                            if other.color != color {
                                moves.insert(to);
                            }
                            break;
                        } else {
                            moves.insert(to);
                        }
                    }
                };
                let mut moves = BTreeSet::new();
                match piece {
                    PieceType::Pawn => match color {
                        // TODO: promotion, en passant
                        PieceColor::White => {
                            if self.board.get(&from.up()).is_none() {
                                moves.insert(from.up());
                            }
                            if from.rank == 1 && self.board.get(&from.up().up()).is_none() {
                                moves.insert(from.up().up());
                            }
                            if self
                                .board
                                .get(&from.up().right())
                                .is_some_and(|other| other.color == PieceColor::Black)
                            {
                                moves.insert(from.up().right());
                            }
                            if self
                                .board
                                .get(&from.up().left())
                                .is_some_and(|other| other.color == PieceColor::Black)
                            {
                                moves.insert(from.up().left());
                            }
                        }
                        PieceColor::Black => {
                            if self.board.get(&from.down()).is_none() {
                                moves.insert(from.down());
                            }
                            if from.rank == 6 && self.board.get(&from.down().down()).is_none() {
                                moves.insert(from.down().down());
                            }
                            if self
                                .board
                                .get(&from.down().right())
                                .is_some_and(|other| other.color == PieceColor::White)
                            {
                                moves.insert(from.down().right());
                            }
                            if self
                                .board
                                .get(&from.down().left())
                                .is_some_and(|other| other.color == PieceColor::White)
                            {
                                moves.insert(from.down().left());
                            }
                        }
                    },
                    PieceType::Knight => {
                        try_insert(&mut moves, from.up().up().left());
                        try_insert(&mut moves, from.up().up().right());
                        try_insert(&mut moves, from.left().left().up());
                        try_insert(&mut moves, from.left().left().down());
                        try_insert(&mut moves, from.down().down().left());
                        try_insert(&mut moves, from.down().down().right());
                        try_insert(&mut moves, from.right().right().up());
                        try_insert(&mut moves, from.right().right().down());
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
                        try_insert(&mut moves, from.up());
                        try_insert(&mut moves, from.up().right());
                        try_insert(&mut moves, from.right());
                        try_insert(&mut moves, from.down().right());
                        try_insert(&mut moves, from.down());
                        try_insert(&mut moves, from.down().left());
                        try_insert(&mut moves, from.left());
                        try_insert(&mut moves, from.up().left());
                    }
                }
                (from, moves)
            })
            .collect()
    }

    fn moves(&self, color: PieceColor) -> BTreeMap<Position, BTreeSet<Move>> {
        let mut moves = self.potential_moves(color);
        for (&from, moves) in &mut moves {
            // NOTE: may violate the precondition of self.move that the move is not a promotion.
            // Left this way, because the specific piece we promote to doesn't affect check and is currently unknown.
            moves.retain(|&to| !self.r#move(from, to).check(color));
        }
        moves
    }

    /// REQUIRES: there is a piece at `from` and move is not a promotion.
    /// If the move is a promotion, use `promote` instead.
    fn r#move(&self, from: Position, to: Move) -> Self {
        // debug_assert!(!self.is_promotion(from, to), "{from}->{to}"); -- intentionally violated when checking for check
        let turn = !self.turn;
        let board = self.board.r#move(from, to);
        Self { turn, board }
    }

    /// REQUIRES: there is a piece at `from` and move is a promotion.
    fn promote(&self, from: Position, to: Move, piece_type: PieceType) -> Self {
        let turn = !self.turn;
        let board = self.board.promote(from, to, piece_type);
        Self { turn, board }
    }

    fn is_promotion(&self, from: Position, to: Move) -> bool {
        let Some(&Piece {
            color,
            piece: PieceType::Pawn,
        }) = self.board.0.get(&from)
        else {
            return false;
        };
        matches!(
            (color, to.rank),
            (PieceColor::White, 7) | (PieceColor::Black, 0)
        )
    }

    fn check(&self, color: PieceColor) -> bool {
        let king = Piece {
            piece: PieceType::King,
            color,
        };
        let king_position = self.board.position_of(king).expect("king always exists");
        self.potential_moves(!color)
            .values()
            .any(|moves| moves.contains(&king_position))
    }

    fn mate(&self, color: PieceColor) -> bool {
        self.moves(color).values().all(|moves| moves.is_empty())
    }

    /// returns None if the game is still in progress
    fn status(&self) -> Option<Outcome> {
        self.mate(self.turn).then(|| {
            if self.check(self.turn) {
                Outcome::Win(!self.turn)
            } else {
                Outcome::Draw
            }
        })
    }
}

struct Tui {
    game: Game,
    click_targets: Vec<(Rect, Position)>,
    selected_tile: Option<(Position, BTreeSet<Position>)>,
    selected_promotion: Option<(
        Position,
        Position,
        BTreeMap<ratatui::layout::Position, PieceType>,
    )>,
}

impl Tui {
    fn new() -> Self {
        let game = Game::new();
        let click_targets = Vec::new();
        let selected_tile = None;
        let selected_promotion = None;
        Self {
            game,
            click_targets,
            selected_tile,
            selected_promotion,
        }
    }

    fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> std::io::Result<Option<Outcome>> {
        let outcome = loop {
            if let Some(outcome) = self.game.status() {
                break Some(outcome);
            }
            terminal.draw(|frame| frame.render_widget(&mut *self, frame.area()))?;
            let event = event::read()?;
            if self.handle(event) {
                break None;
            }
        };
        return Ok(outcome);
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
                if let Some((from, to, ref click_targets)) = self.selected_promotion {
                    if let Some(&piece_type) = click_targets.get(&click) {
                        self.game = self.game.promote(from, to, piece_type);
                        self.selected_promotion = None;
                        return false;
                    }
                }
                for &(rect, position) in &self.click_targets {
                    if rect.contains(click) {
                        match self.selected_tile {
                            None => self.select_tile(position),
                            Some((from, ref moves)) => {
                                if moves.contains(&position) {
                                    if self.game.is_promotion(from, position) {
                                        // promotion click targets will be populated upon rendering
                                        self.selected_promotion =
                                            Some((from, position, BTreeMap::new()));
                                    } else {
                                        self.game = self.game.r#move(from, position);
                                    }
                                    self.selected_tile = None;
                                } else {
                                    self.select_tile(position);
                                }
                            }
                        }
                        return false;
                    }
                }
                false
            }
            _ => false,
        }
    }

    fn select_tile(&mut self, position: Position) {
        self.selected_tile = self.game.moves(self.game.turn).remove_entry(&position);
        self.selected_promotion = None;
    }
}

impl Widget for &mut Tui {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let [area] = Layout::vertical([8]).flex(Flex::Center).areas(area);
        let [board_area, info_area] = Layout::horizontal([8 * 2, 5])
            .spacing(1)
            .flex(Flex::Center)
            .areas(area);

        // board
        let ranks = Layout::vertical([Constraint::Fill(1); 8]).split(board_area);
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
                    || self
                        .selected_promotion
                        .as_ref()
                        .is_some_and(|(from, to, _)| *from == position || *to == position)
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

        // info
        let [black_turn_area, promotion_area, white_turn_area] = Layout::vertical([2, 1, 2])
            .flex(Flex::SpaceBetween)
            .areas(info_area);
        let turn_area = match self.game.turn {
            PieceColor::White => white_turn_area,
            PieceColor::Black => black_turn_area,
        };
        let mut text = Text::default();
        let turn_span = Span::raw(self.game.turn.to_string())
            .fg(self.game.turn.render())
            .bg(Color::Gray);
        text.push_span(turn_span);
        if self.game.check(self.game.turn) {
            let check_line = Line::raw("check").bg(Color::LightRed).fg(Color::Gray);
            text.push_line(check_line);
        }
        text.render(turn_area, buf);

        // promotion
        if let Some((_, _, click_targets)) = &mut self.selected_promotion {
            click_targets.clear();
            for (area, piece) in promotion_area.columns().zip([
                PieceType::Queen,
                PieceType::Rook,
                PieceType::Bishop,
                PieceType::Knight,
            ]) {
                piece
                    .render()
                    .fg(self.game.turn.render())
                    .bg(Color::Gray)
                    .render(area, buf);
                click_targets.insert(area.as_position(), piece);
            }
        }
    }
}

fn main() -> ExitCode {
    let mut tui = Tui::new();
    let mut terminal = ratatui::init();
    terminal
        .backend_mut()
        .writer_mut()
        .execute(EnableMouseCapture)
        .expect("enable mouse capture");
    let result = tui.run(&mut terminal);
    terminal
        .backend_mut()
        .writer_mut()
        .execute(DisableMouseCapture)
        .expect("disable mouse capture");
    ratatui::restore();
    match result {
        Ok(outcome) => {
            if let Some(outcome) = outcome {
                println!("Outcome: {outcome}");
            } else {
                println!("Quit before game ended");
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("ERROR: {e}");
            ExitCode::FAILURE
        }
    }
}
