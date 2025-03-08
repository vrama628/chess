use std::ops::{Index, IndexMut};

use crate::game::piece::PieceColor;

#[derive(Clone, Copy)]
pub enum CastlingInfo {
    KingHasNotMoved {
        queenside_rook_has_not_moved: bool,
        kingside_rook_has_not_moved: bool,
    },
    KingHasMoved,
}

// TODO: make updates return Self instead of mutate once I switch to persistent data structures
impl CastlingInfo {
    fn new() -> Self {
        Self::KingHasNotMoved {
            queenside_rook_has_not_moved: true,
            kingside_rook_has_not_moved: true,
        }
    }

    pub fn move_king(&mut self) {
        *self = Self::KingHasMoved
    }

    pub fn move_queenside_rook(&mut self) {
        if let Self::KingHasNotMoved {
            queenside_rook_has_not_moved,
            ..
        } = self
        {
            *queenside_rook_has_not_moved = false;
        }
    }

    pub fn move_kingside_rook(&mut self) {
        if let Self::KingHasNotMoved {
            kingside_rook_has_not_moved,
            ..
        } = self
        {
            *kingside_rook_has_not_moved = false;
        }
    }

    pub fn can_castle_queenside(&self) -> bool {
        matches!(
            self,
            Self::KingHasNotMoved {
                queenside_rook_has_not_moved: true,
                ..
            }
        )
    }

    pub fn can_castle_kingside(&self) -> bool {
        matches!(
            self,
            Self::KingHasNotMoved {
                kingside_rook_has_not_moved: true,
                ..
            }
        )
    }
}

#[derive(Clone, Copy)]
pub struct Castling {
    white: CastlingInfo,
    black: CastlingInfo,
}

impl Castling {
    pub fn new() -> Self {
        let white = CastlingInfo::new();
        let black = CastlingInfo::new();
        Self { white, black }
    }
}

impl Index<PieceColor> for Castling {
    type Output = CastlingInfo;

    fn index(&self, color: PieceColor) -> &Self::Output {
        match color {
            PieceColor::White => &self.white,
            PieceColor::Black => &self.black,
        }
    }
}

impl IndexMut<PieceColor> for Castling {
    fn index_mut(&mut self, color: PieceColor) -> &mut Self::Output {
        match color {
            PieceColor::White => &mut self.white,
            PieceColor::Black => &mut self.black,
        }
    }
}
