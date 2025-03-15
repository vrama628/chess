use std::{fmt::Display, ops::Not};

use ratatui::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PieceType {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

impl PieceType {
    pub fn render(&self) -> &'static str {
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

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum PieceColor {
    White,
    Black,
}

impl PieceColor {
    pub fn render(&self) -> Color {
        match self {
            Self::White => Color::White,
            Self::Black => Color::Black,
        }
    }

    pub fn pawn_starting_rank(&self) -> u8 {
        match self {
            Self::White => 1,
            Self::Black => 6,
        }
    }

    pub fn piece_starting_rank(&self) -> u8 {
        match self {
            Self::White => 0,
            Self::Black => 7,
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

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Piece {
    pub color: PieceColor,
    pub piece: PieceType,
}

impl Piece {
    pub fn render(&self) -> Span<'static> {
        Span::raw(self.piece.render()).fg(self.color.render())
    }
}
