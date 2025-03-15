use std::fmt::Display;

use ratatui::prelude::*;

use crate::game::piece::PieceColor;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub struct Position(pub(super) u8);

impl Position {
    pub fn new(rank: u8, file: u8) -> Self {
        debug_assert!(rank < 8);
        debug_assert!(file < 8);
        Self(rank << 3 | file)
    }

    pub fn rank(self) -> u8 {
        self.0 >> 3
    }

    pub fn file(self) -> u8 {
        self.0 & 0b111
    }

    pub fn square(self) -> Line<'static> {
        let bg = if (self.rank() + self.file()) % 2 == 0 {
            Color::DarkGray
        } else {
            Color::Gray
        };
        Line::default().bg(bg)
    }
}

impl Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", (b'a' + self.file()) as char, self.rank() + 1)
    }
}

pub trait Movement {
    fn up(self) -> Option<Position>;
    fn down(self) -> Option<Position>;
    fn left(self) -> Option<Position>;
    fn right(self) -> Option<Position>;
    fn pawn(self, color: PieceColor) -> Option<Position>;
}

impl Movement for Position {
    fn up(self) -> Option<Self> {
        (self.rank() < 7).then(|| Self(self.0 + (1 << 3)))
    }

    fn down(self) -> Option<Self> {
        (self.rank() > 0).then(|| Self(self.0 - (1 << 3)))
    }

    fn left(self) -> Option<Self> {
        (self.file() > 0).then(|| Self(self.0 - 1))
    }

    fn right(self) -> Option<Self> {
        (self.file() < 7).then(|| Self(self.0 + 1))
    }

    fn pawn(self, color: PieceColor) -> Option<Self> {
        match color {
            PieceColor::White => self.up(),
            PieceColor::Black => self.down(),
        }
    }
}

impl Movement for Option<Position> {
    fn up(self) -> Self {
        self.and_then(Movement::up)
    }

    fn down(self) -> Self {
        self.and_then(Movement::down)
    }

    fn left(self) -> Self {
        self.and_then(Movement::left)
    }

    fn right(self) -> Self {
        self.and_then(Movement::right)
    }

    fn pawn(self, color: PieceColor) -> Self {
        self.and_then(|position| position.pawn(color))
    }
}
