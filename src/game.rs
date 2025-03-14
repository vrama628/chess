use std::fmt::Display;

mod board;
mod castling;
pub mod piece;
pub mod position;

use board::Board;
use castling::Castling;
use piece::{Piece, PieceColor, PieceType};
use position::Position;

pub struct Game {
    turn: PieceColor,
    board: Board,
    /// for en passant
    just_advanced_two: Option<Position>,
    /// for castling
    castling: Castling,
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Outcome {
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

pub const PROMOTIONS: [PieceType; 4] = [
    PieceType::Queen,
    PieceType::Rook,
    PieceType::Bishop,
    PieceType::Knight,
];

impl Game {
    pub fn new() -> Self {
        let turn = PieceColor::White;
        let board = Board::new();
        let just_advanced_two = None;
        let castling = Castling::new();
        Self {
            turn,
            board,
            just_advanced_two,
            castling,
        }
    }

    pub fn turn(&self) -> PieceColor {
        self.turn
    }

    pub fn get(&self, position: Position) -> Option<&Piece> {
        self.board.get(position)
    }

    /// returns moves that can be made, but without filtering out moves into check
    /// ENSURES: there is a piece at all keys of the returned map
    pub fn moves(&self, color: PieceColor) -> impl Iterator<Item = (Position, Vec<Position>)> + '_ {
        let king_position = {
            let king = Piece {
                piece: PieceType::King,
                color,
            };
            self.board.position_of(king).expect("king always exists")
        };
        self.board
            .iter(color)
            .map(move |(&from, &Piece { color, piece })| {
                let does_not_cause_check = |to: Position| {
                    let after_move = if cfg!(debug_assertions) && self.is_promotion(from, to) {
                        self.promote(from, to, PieceType::Queen)
                    } else {
                        self.r#move(from, to)
                    };
                    !after_move.attacks(!color, king_position)
                };
                let mut moves = vec![];
                let mut saturate = |f: &dyn Fn(Position) -> Position| {
                    let mut to = f(from);
                    while to.is_valid() {
                        if let Some(other) = self.board.get(to) {
                            if other.color != color && does_not_cause_check(to) {
                                moves.push(to);
                            }
                            break;
                        } else if does_not_cause_check(to) {
                            moves.push(to);
                        }
                        to = f(to);
                    }
                };
                match piece {
                    PieceType::Pawn => {
                        let forward = from.pawn(color);
                        if self.board.is_vacant(forward) && does_not_cause_check(forward) {
                            moves.push(forward);
                        }

                        let forward_two = forward.pawn(color);
                        if from.rank == color.pawn_starting_rank()
                            && self.board.is_vacant(forward)
                            && self.board.is_vacant(forward_two)
                            && does_not_cause_check(forward_two)
                        {
                            moves.push(forward_two);
                        }

                        let capture_left = forward.left();
                        if (self
                            .board
                            .get(capture_left)
                            .is_some_and(|other| other.color == !color)
                            || self.just_advanced_two.is_some_and(|position| {
                                // en passant
                                position == from.left()
                                    && self
                                        .board
                                        .get(position)
                                        .expect("Game::just_advanced_two invariant")
                                        .color
                                        == !color
                            }))
                            && does_not_cause_check(capture_left)
                        {
                            moves.push(capture_left);
                        }

                        let capture_right = forward.right();
                        if (self
                            .board
                            .get(capture_right)
                            .is_some_and(|other| other.color == !color)
                            || self.just_advanced_two.is_some_and(|position| {
                                // en passant
                                position == from.right()
                                    && self
                                        .board
                                        .get(position)
                                        .expect("Game::just_advanced_two invariant")
                                        .color
                                        == !color
                            }))
                            && does_not_cause_check(capture_right)
                        {
                            moves.push(capture_right);
                        }
                    }
                    PieceType::Knight => {
                        let mut try_insert = |to: Position| {
                            if to.is_valid()
                                && self.board.get(to).is_none_or(|other| other.color != color)
                                && does_not_cause_check(to)
                            {
                                moves.push(to);
                            }
                        };
                        try_insert(from.up().up().left());
                        try_insert(from.up().up().right());
                        try_insert(from.left().left().up());
                        try_insert(from.left().left().down());
                        try_insert(from.down().down().left());
                        try_insert(from.down().down().right());
                        try_insert(from.right().right().up());
                        try_insert(from.right().right().down());
                    }
                    PieceType::Bishop => {
                        saturate(&|p| p.up().left());
                        saturate(&|p| p.up().right());
                        saturate(&|p| p.down().left());
                        saturate(&|p| p.down().right());
                    }
                    PieceType::Rook => {
                        saturate(&|p| p.up());
                        saturate(&|p| p.left());
                        saturate(&|p| p.down());
                        saturate(&|p| p.right());
                    }
                    PieceType::Queen => {
                        saturate(&|p| p.up());
                        saturate(&|p| p.left());
                        saturate(&|p| p.down());
                        saturate(&|p| p.right());
                        saturate(&|p| p.up().left());
                        saturate(&|p| p.up().right());
                        saturate(&|p| p.down().left());
                        saturate(&|p| p.down().right());
                    }
                    PieceType::King => {
                        let does_not_cause_check =
                            |to: Position| !self.r#move(from, to).attacks(!color, to);
                        let mut try_insert = |to: Position| {
                            if to.is_valid()
                                && self.board.get(to).is_none_or(|other| other.color != color)
                                && does_not_cause_check(to)
                            {
                                moves.push(to);
                            }
                        };
                        try_insert(from.up());
                        try_insert(from.up().right());
                        try_insert(from.right());
                        try_insert(from.down().right());
                        try_insert(from.down());
                        try_insert(from.down().left());
                        try_insert(from.left());
                        try_insert(from.up().left());

                        // queenside castling
                        if self.castling[color].can_castle_queenside()
                            && self.board.is_vacant(from.left())
                            && self.board.is_vacant(from.left().left())
                            && self.board.is_vacant(from.left().left().left())
                            && !self.attacks(!color, from)
                            && !self.attacks(!color, from.left())
                            && !self.attacks(!color, from.left().left())
                        {
                            moves.push(from.left().left());
                        }

                        // kingside castling
                        if self.castling[color].can_castle_kingside()
                            && self.board.is_vacant(from.right())
                            && self.board.is_vacant(from.right().right())
                            && !self.attacks(!color, from)
                            && !self.attacks(!color, from.right())
                            && !self.attacks(!color, from.right().right())
                        {
                            moves.push(from.right().right());
                        }
                    }
                }
                (from, moves)
            })
    }

    fn bishop_attacks(&self, position: Position, target: Position) -> bool {
        position.rank.abs_diff(target.rank) == position.file.abs_diff(target.file)
            && (position.rank.min(target.rank)..position.rank.max(target.rank))
                .zip(position.file.min(target.file)..position.file.max(target.file))
                .skip(1)
                .all(|(rank, file)| self.get(Position { rank, file }).is_none())
    }

    fn rook_attacks(&self, position: Position, target: Position) -> bool {
        (position.rank == target.rank
            && (position.file.min(target.file)..position.file.max(target.file))
                .skip(1)
                .all(|file| {
                    self.get(Position {
                        rank: position.rank,
                        file,
                    })
                    .is_none()
                }))
            || (position.file == target.file
                && (position.rank.min(target.rank)..position.rank.max(target.rank))
                    .skip(1)
                    .all(|rank| {
                        self.get(Position {
                            rank,
                            file: position.file,
                        })
                        .is_none()
                    }))
    }

    fn attacks(&self, color: PieceColor, target: Position) -> bool {
        self.board
            .iter(color)
            .any(|(&position, piece)| match piece.piece {
                PieceType::Pawn => {
                    let pawn_move = position.pawn(color);
                    pawn_move.rank == target.rank && pawn_move.file.abs_diff(target.file) == 1
                }
                PieceType::Knight => matches!(
                    (
                        position.rank.abs_diff(target.rank),
                        position.file.abs_diff(target.file),
                    ),
                    (1, 2) | (2, 1)
                ),
                PieceType::Bishop => self.bishop_attacks(position, target),
                PieceType::Rook => self.rook_attacks(position, target),
                PieceType::Queen => {
                    self.bishop_attacks(position, target) || self.rook_attacks(position, target)
                }
                PieceType::King => {
                    position.rank.abs_diff(target.rank) <= 1
                        && position.file.abs_diff(target.file) <= 1
                }
            })
    }

    /// REQUIRES: there is a piece at `from` and move is not a promotion.
    /// If the move is a promotion, use `promote` instead.
    pub fn r#move(&self, from: Position, to: Position) -> Self {
        debug_assert!(!self.is_promotion(from, to), "{from} -> {to}");
        let piece = self.board.get(from).expect("Game::move precondition");
        let turn = !self.turn;
        let board = self.board.r#move(from, to);
        let just_advanced_two =
            (piece.piece == PieceType::Pawn && from.rank.abs_diff(to.rank) == 2).then(|| to);
        let mut castling = self.castling;
        if from.rank == piece.color.piece_starting_rank() {
            match (piece.piece, from.file) {
                (PieceType::King, 4) => {
                    castling[piece.color].move_king();
                }
                (PieceType::Rook, 0) => {
                    castling[piece.color].move_queenside_rook();
                }
                (PieceType::Rook, 7) => {
                    castling[piece.color].move_kingside_rook();
                }
                _ => {}
            }
        }
        Self {
            turn,
            board,
            just_advanced_two,
            castling,
        }
    }

    /// REQUIRES: there is a pawn at `from` and move is a promotion.
    pub fn promote(&self, from: Position, to: Position, piece_type: PieceType) -> Self {
        debug_assert!(self.is_promotion(from, to), "{from} -> {to}");
        debug_assert!(PROMOTIONS.contains(&piece_type), "{piece_type:?}");
        let turn = !self.turn;
        let board = self.board.promote(from, to, piece_type);
        let just_advanced_two = None;
        let castling_info = self.castling.clone();
        Self {
            turn,
            board,
            just_advanced_two,
            castling: castling_info,
        }
    }

    pub fn is_promotion(&self, from: Position, to: Position) -> bool {
        let Some(&Piece {
            color,
            piece: PieceType::Pawn,
        }) = self.board.get(from)
        else {
            return false;
        };
        matches!(
            (color, to.rank),
            (PieceColor::White, 7) | (PieceColor::Black, 0)
        )
    }

    pub fn check(&self, color: PieceColor) -> bool {
        let king = Piece {
            piece: PieceType::King,
            color,
        };
        let king_position = self.board.position_of(king).expect("king always exists");
        self.attacks(!color, king_position)
    }

    fn mate(&self, color: PieceColor) -> bool {
        self.moves(color).all(|(_, moves)| moves.is_empty())
    }

    /// returns None if the game is still in progress
    pub fn status(&self) -> Option<Outcome> {
        self.mate(self.turn).then(|| {
            if self.check(self.turn) {
                // mate is check
                Outcome::Win(!self.turn)
            } else {
                // mate is stale
                Outcome::Draw
            }
        })
    }

    pub fn iter(&self, color: PieceColor) -> impl Iterator<Item = (&Position, &Piece)> {
        self.board.iter(color)
    }
}
