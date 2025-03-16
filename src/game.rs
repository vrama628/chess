use std::fmt::{Debug, Display};

mod board;
mod castling;
pub mod piece;

pub use board::position::Position;
use board::{position::Movement, Board};
use castling::Castling;
pub use piece::{Piece, PieceColor, PieceType};

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

    pub fn get(&self, position: Position) -> Option<Piece> {
        self.board.get(position)
    }

    /// returns moves that can be made, but without filtering out moves into check
    /// ENSURES: there is a piece at all keys of the returned map
    pub fn moves(&self) -> impl Iterator<Item = (Position, Vec<Position>)> + '_ {
        let king_position = {
            let king = Piece {
                piece: PieceType::King,
                color: self.turn,
            };
            self.board.position_of(king).expect("king always exists")
        };
        self.board
            .iter(self.turn)
            .map(move |(from, Piece { piece, color })| {
                debug_assert_eq!(color, self.turn);
                let does_not_cause_check = |to: Position| {
                    let after_move = if cfg!(debug_assertions) && self.is_promotion(from, to) {
                        self.promote(from, to, PieceType::Queen)
                    } else {
                        self.r#move(from, to)
                    };
                    !after_move.attacks(!color, king_position)
                };
                let mut moves = vec![];
                let mut saturate = |f: &dyn Fn(Position) -> Option<Position>| {
                    let mut to_opt = f(from);
                    while let Some(to) = to_opt {
                        if let Some(other) = self.board.get(to) {
                            if other.color != color && does_not_cause_check(to) {
                                moves.push(to);
                            }
                            break;
                        } else if does_not_cause_check(to) {
                            moves.push(to);
                        }
                        to_opt = f(to);
                    }
                };
                match piece {
                    PieceType::Pawn => {
                        let forward = from.pawn(color).expect("pawn is never on last rank");
                        if self.board.is_vacant(forward) && does_not_cause_check(forward) {
                            moves.push(forward);
                        }

                        if let Some(forward_two) = forward.pawn(color) {
                            if from.rank() == color.pawn_starting_rank()
                                && self.board.is_vacant(forward)
                                && self.board.is_vacant(forward_two)
                                && does_not_cause_check(forward_two)
                            {
                                moves.push(forward_two);
                            }
                        }

                        if let Some(capture_left) = forward.left() {
                            if (self
                                .board
                                .get(capture_left)
                                .is_some_and(|other| other.color == !color)
                                || self.just_advanced_two.is_some_and(|position| {
                                    // en passant
                                    position == from.left().expect("rectangle")
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
                        }

                        if let Some(capture_right) = forward.right() {
                            if (self
                                .board
                                .get(capture_right)
                                .is_some_and(|other| other.color == !color)
                                || self.just_advanced_two.is_some_and(|position| {
                                    // en passant
                                    position == from.right().expect("rectangle")
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
                    }
                    PieceType::Knight => {
                        let mut try_insert = |to: Option<Position>| {
                            if let Some(to) = to {
                                if self.board.get(to).is_none_or(|other| other.color != color)
                                    && does_not_cause_check(to)
                                {
                                    moves.push(to);
                                }
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
                        let mut try_insert = |to: Option<Position>| {
                            if let Some(to) = to {
                                if self.board.get(to).is_none_or(|other| other.color != color)
                                    && does_not_cause_check(to)
                                {
                                    moves.push(to);
                                }
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

                        if self.castling[color].can_castle_queenside() {
                            let left = from.left().expect("castling");
                            let left_left = left.left().expect("castling");
                            let left_left_left = left_left.left().expect("castling");
                            if self.board.is_vacant(left)
                                && self.board.is_vacant(left_left)
                                && self.board.is_vacant(left_left_left)
                                && !self.attacks(!color, from)
                                && !self.attacks(!color, left)
                                && !self.attacks(!color, left_left)
                            {
                                moves.push(left_left);
                            }
                        }

                        if self.castling[color].can_castle_kingside() {
                            let right = from.right().expect("castling");
                            let right_right = right.right().expect("castling");
                            if self.board.is_vacant(right)
                                && self.board.is_vacant(right_right)
                                && !self.attacks(!color, from)
                                && !self.attacks(!color, right)
                                && !self.attacks(!color, right_right)
                            {
                                moves.push(right_right);
                            }
                        }
                    }
                }
                (from, moves)
            })
    }

    fn sliding_attacks(&self, mut position: Position, target: Position) -> bool {
        let d_rank = target.rank().cmp(&position.rank());
        let d_file = target.file().cmp(&position.file());
        loop {
            position = Position::new(
                ((position.rank() as i8) + (d_rank as i8)) as u8,
                ((position.file() as i8) + (d_file as i8)) as u8,
            );
            if position == target {
                return true;
            }
            if !self.board.is_vacant(position) {
                return false;
            }
        }
    }

    fn attacks(&self, color: PieceColor, target: Position) -> bool {
        self.board
            .iter(color)
            .any(|(position, piece)| match piece.piece {
                PieceType::Pawn => {
                    let pawn_move = position.pawn(color).expect("pawn not on last rank");
                    pawn_move.rank() == target.rank()
                        && pawn_move.file().abs_diff(target.file()) == 1
                }
                PieceType::Knight => matches!(
                    (
                        position.rank().abs_diff(target.rank()),
                        position.file().abs_diff(target.file()),
                    ),
                    (1, 2) | (2, 1)
                ),
                PieceType::Bishop => {
                    position.rank().abs_diff(target.rank())
                        == position.file().abs_diff(target.file())
                        && self.sliding_attacks(position, target)
                }
                PieceType::Rook => {
                    (position.rank() == target.rank() || position.file() == target.file())
                        && self.sliding_attacks(position, target)
                }
                PieceType::Queen => {
                    (position.rank().abs_diff(target.rank())
                        == position.file().abs_diff(target.file())
                        || position.rank() == target.rank()
                        || position.file() == target.file())
                        && self.sliding_attacks(position, target)
                }
                PieceType::King => {
                    position.rank().abs_diff(target.rank()) <= 1
                        && position.file().abs_diff(target.file()) <= 1
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
            (piece.piece == PieceType::Pawn && from.rank().abs_diff(to.rank()) == 2).then(|| to);
        let mut castling = self.castling;
        if from.rank() == piece.color.piece_starting_rank() {
            match (piece.piece, from.file()) {
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
        let Some(Piece {
            piece: PieceType::Pawn,
            color,
        }) = self.board.get(from)
        else {
            return false;
        };
        matches!(
            (color, to.rank()),
            (PieceColor::White, 7) | (PieceColor::Black, 0)
        )
    }

    pub fn check(&self) -> bool {
        let king = Piece {
            piece: PieceType::King,
            color: self.turn,
        };
        let king_position = self.board.position_of(king).expect("king always exists");
        self.attacks(!self.turn, king_position)
    }

    fn mate(&self) -> bool {
        self.moves().all(|(_, moves)| moves.is_empty())
    }

    /// returns None if the game is still in progress
    pub fn status(&self) -> Option<Outcome> {
        self.mate().then(|| {
            if self.check() {
                // mate is check
                Outcome::Win(!self.turn)
            } else {
                // mate is stale
                Outcome::Draw
            }
        })
    }

    pub fn iter(&self, color: PieceColor) -> impl Iterator<Item = (Position, Piece)> + '_ {
        self.board.iter(color)
    }
}

impl Debug for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{} to move", self.turn)?;
        write!(f, "{:?}", self.board)
    }
}

#[cfg(test)]
mod test {
    use super::{piece::PieceType, Game};

    #[test]
    fn moves_and_attacks_are_consistent() {
        fn rec(game: &Game, depth: usize) {
            if depth == 0 || game.status().is_some() {
                return;
            }
            for (from, moves) in game.moves() {
                assert!(game.get(from).is_some_and(|piece| piece.color == game.turn));
                for to in moves {
                    if let Some(capture) = game.get(to) {
                        assert_eq!(capture.color, !game.turn, "capture of wrong color");
                        assert!(game.attacks(game.turn, to), "Game::moves sees capture {from}->{to} but Game::attacks does not:\n{game:?}")
                    }
                    let game = if game.is_promotion(from, to) {
                        game.promote(from, to, PieceType::Queen)
                    } else {
                        game.r#move(from, to)
                    };
                    rec(&game, depth - 1)
                }
            }
        }
        rec(&Game::new(), 3)
    }
}
