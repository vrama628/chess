use std::fmt::Display;

use ratatui::prelude::*;

use crate::game::piece::PieceColor;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub struct Position {
    pub rank: usize,
    pub file: usize,
}

impl Position {
    pub fn square(&self) -> Line<'static> {
        let bg = if (self.rank + self.file) % 2 == 0 {
            Color::DarkGray
        } else {
            Color::Gray
        };
        Line::default().bg(bg)
    }

    pub fn up(&self) -> Self {
        Self {
            rank: self.rank + 1,
            file: self.file,
        }
    }

    pub fn down(&self) -> Self {
        Self {
            rank: self.rank.wrapping_sub(1),
            file: self.file,
        }
    }

    pub fn left(&self) -> Self {
        Self {
            rank: self.rank,
            file: self.file.wrapping_sub(1),
        }
    }

    pub fn right(&self) -> Self {
        Self {
            rank: self.rank,
            file: self.file + 1,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.rank < 8 && self.file < 8
    }

    pub fn pawn(&self, color: PieceColor) -> Self {
        match color {
            PieceColor::White => self.up(),
            PieceColor::Black => self.down(),
        }
    }
}

impl Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", (b'a' + self.file as u8) as char, self.rank + 1)
    }
}
