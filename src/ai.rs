use std::cmp::Ordering;

use crate::game::{
    piece::{PieceColor, PieceType},
    position::Position,
    Game, Outcome, PROMOTIONS,
};

pub enum Move {
    Move(Position, Position),
    Promote(Position, Position, PieceType),
}

#[derive(PartialEq, Eq)]
enum Evaluation {
    Outcome(Outcome),
    Estimate(i8),
}

impl Ord for Evaluation {
    fn cmp(&self, other: &Self) -> Ordering {
        use self::Outcome::*;
        use Evaluation::*;
        use PieceColor::*;
        match (self, other) {
            (Outcome(Win(White)), Outcome(Win(White))) => Ordering::Equal,
            (Outcome(Win(White)), _) => Ordering::Greater,
            (_, Outcome(Win(White))) => Ordering::Less,
            (Outcome(Win(Black)), Outcome(Win(Black))) => Ordering::Equal,
            (Outcome(Win(Black)), _) => Ordering::Less,
            (_, Outcome(Win(Black))) => Ordering::Greater,
            (Outcome(Draw), Outcome(Draw)) => Ordering::Equal,
            (Outcome(Draw), Estimate(n)) => 0.cmp(n),
            (Estimate(n), Outcome(Draw)) => n.cmp(&0),
            (Estimate(n), Estimate(m)) => n.cmp(m),
        }
    }
}

impl PartialOrd for Evaluation {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn value(piece: PieceType) -> i8 {
    match piece {
        PieceType::Pawn => 1,
        PieceType::Knight => 3,
        PieceType::Bishop => 3,
        PieceType::Rook => 5,
        PieceType::Queen => 9,
        PieceType::King => 0,
    }
}

fn estimate(game: Game) -> i8 {
    game.iter(PieceColor::White)
        .map(|(_, piece)| value(piece.piece))
        .sum::<i8>()
        - game
            .iter(PieceColor::Black)
            .map(|(_, piece)| value(piece.piece))
            .sum::<i8>()
}

/// REQUIRES: game is not in mate
fn minimax(game: &Game, depth: usize) -> (Move, Evaluation) {
    let mut best: Option<(Move, Evaluation)> = None;
    let better = match game.turn() {
        PieceColor::White => Ordering::Greater,
        PieceColor::Black => Ordering::Less,
    };
    for r#move in game
        .moves(game.turn())
        .into_iter()
        .flat_map(|(from, to)| to.into_iter().map(move |to| (from, to)))
        .flat_map(|(from, to)| {
            if game.is_promotion(from, to) {
                PROMOTIONS
                    .into_iter()
                    .map(|piece_type| Move::Promote(from, to, piece_type))
                    .collect()
            } else {
                vec![Move::Move(from, to)]
            }
        })
    {
        let game = match r#move {
            Move::Move(from, to) => game.r#move(from, to),
            Move::Promote(from, to, piece_type) => game.promote(from, to, piece_type),
        };
        let evaluation = if let Some(outcome) = game.status() {
            Evaluation::Outcome(outcome)
        } else if depth == 0 {
            Evaluation::Estimate(estimate(game))
        } else {
            minimax(&game, depth - 1).1
        };
        if best
            .as_ref()
            .is_none_or(|(_, best_so_far)| evaluation.cmp(best_so_far) == better)
        {
            best = Some((r#move, evaluation))
        }
    }
    best.expect("minimax precondition")
}

/// REQUIRES: game is not in mate
pub fn choose(game: &Game, depth: usize) -> Move {
    minimax(game, depth).0
}
