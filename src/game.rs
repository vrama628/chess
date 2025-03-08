use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Display,
};

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

    /// TODO: castling
    /// returns moves that can be made, but without filtering out moves into check
    /// ENSURES: there is a piece at all keys of the returned map
    fn potential_moves(&self, color: PieceColor) -> BTreeMap<Position, BTreeSet<Position>> {
        self.board
            .iter(color)
            .map(|(&from, &Piece { color, piece })| {
                let try_insert = |moves: &mut BTreeSet<Position>, to: Position| {
                    if to.is_valid() && self.board.get(to).is_none_or(|other| other.color != color)
                    {
                        moves.insert(to);
                    }
                };
                let saturate = |moves: &mut BTreeSet<Position>,
                                f: &dyn Fn(Position) -> Position| {
                    let mut to = from;
                    while to.is_valid() {
                        to = f(to);
                        if let Some(other) = self.board.get(to) {
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
                    PieceType::Pawn => {
                        let forward = from.pawn(color);
                        if self.board.is_vacant(forward) {
                            moves.insert(forward);
                        }

                        let forward_two = forward.pawn(color);
                        if from.rank == color.pawn_starting_rank()
                            && self.board.is_vacant(forward)
                            && self.board.is_vacant(forward_two)
                        {
                            moves.insert(forward_two);
                        }

                        let capture_left = forward.left();
                        if self
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
                            })
                        {
                            moves.insert(capture_left);
                        }

                        let capture_right = forward.right();
                        if self
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
                            })
                        {
                            moves.insert(capture_right);
                        }
                    }
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
                        try_insert(&mut moves, from.up());
                        try_insert(&mut moves, from.up().right());
                        try_insert(&mut moves, from.right());
                        try_insert(&mut moves, from.down().right());
                        try_insert(&mut moves, from.down());
                        try_insert(&mut moves, from.down().left());
                        try_insert(&mut moves, from.left());
                        try_insert(&mut moves, from.up().left());
                        // castling handled in Game::moves
                    }
                }
                (from, moves)
            })
            .collect()
    }

    /// All possible moves that do not result in check
    pub fn moves(&self, color: PieceColor) -> BTreeMap<Position, BTreeSet<Position>> {
        let mut moves = self.potential_moves(color);
        for (&from, moves) in &mut moves {
            moves.retain(|&to| {
                let after_move = if self.is_promotion(from, to) {
                    self.promote(from, to, PieceType::Queen)
                } else {
                    self.r#move(from, to)
                };
                !after_move.check(color)
            });

            // handle castling here instead of in potential_moves because it requires checking for check,
            // which would infinitely recurse if done in potential_moves, and because castling cannot capture
            // so doesn't need to be included in potential_moves anyway
            if self
                .board
                .get(from)
                .expect("Game::potential_moves postcondition")
                .piece
                == PieceType::King
            {
                // queenside castling
                if self.castling[color].can_castle_queenside()
                    && self.board.is_vacant(from.left())
                    && self.board.is_vacant(from.left().left())
                    && self.board.is_vacant(from.left().left().left())
                    && !self.attacks(!color, from)
                    && !self.attacks(!color, from.left())
                    && !self.attacks(!color, from.left().left())
                {
                    moves.insert(from.left().left());
                }

                // kingside castling
                if self.castling[color].can_castle_kingside()
                    && self.board.is_vacant(from.right())
                    && self.board.is_vacant(from.right().right())
                    && !self.attacks(!color, from)
                    && !self.attacks(!color, from.right())
                    && !self.attacks(!color, from.right().right())
                {
                    moves.insert(from.right().right());
                }
            }
        }
        moves
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
        let mut castling_info = self.castling.clone();
        if from.rank == piece.color.piece_starting_rank() {
            match (piece.piece, from.file) {
                (PieceType::King, 4) => {
                    castling_info[piece.color].move_king();
                }
                (PieceType::Rook, 0) => {
                    castling_info[piece.color].move_queenside_rook();
                }
                (PieceType::Rook, 7) => {
                    castling_info[piece.color].move_kingside_rook();
                }
                _ => {}
            }
        }
        Self {
            turn,
            board,
            just_advanced_two,
            castling: castling_info,
        }
    }

    /// REQUIRES: there is a pawn at `from` and move is a promotion.
    pub fn promote(&self, from: Position, to: Position, piece_type: PieceType) -> Self {
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

    pub fn attacks(&self, color: PieceColor, position: Position) -> bool {
        self.potential_moves(color)
            .values()
            .any(|moves| moves.contains(&position))
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
        self.moves(color).values().all(|moves| moves.is_empty())
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
}
