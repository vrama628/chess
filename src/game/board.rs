pub mod position;

use std::ops::{Index, IndexMut};

use crate::game::piece::{Piece, PieceColor, PieceType};
use position::{Movement, Position};

#[derive(Clone, Copy)]
pub struct Board([Option<Piece>; 64]);

impl Index<Position> for Board {
    type Output = Option<Piece>;

    fn index(&self, position: Position) -> &Self::Output {
        &self.0[position.0 as usize]
    }
}

impl IndexMut<Position> for Board {
    fn index_mut(&mut self, position: Position) -> &mut Self::Output {
        &mut self.0[position.0 as usize]
    }
}

impl Board {
    pub fn new() -> Self {
        let mut this = Self([None; 64]);
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
        for (file, piece) in PIECE_TYPES.into_iter().enumerate() {
            this[Position::new(7, file as u8)] = Some(Piece { color, piece })
        }
        let piece = PieceType::Pawn;
        for file in 0..8 {
            this[Position::new(6, file)] = Some(Piece { color, piece })
        }
        let color = PieceColor::White;
        for file in 0..8 {
            this[Position::new(1, file)] = Some(Piece { color, piece })
        }
        for (file, piece) in PIECE_TYPES.into_iter().enumerate() {
            this[Position::new(0, file as u8)] = Some(Piece { color, piece })
        }
        this
    }

    pub fn get(&self, position: Position) -> Option<Piece> {
        self[position]
    }

    pub fn iter(&self, color: PieceColor) -> impl Iterator<Item = (Position, Piece)> + '_ {
        self.0.iter().enumerate().filter_map(move |(i, piece_opt)| {
            piece_opt
                .filter(|piece| piece.color == color)
                .map(|piece| (Position(i as u8), piece))
        })
    }

    /// REQUIRES: there is a piece at `from`
    pub fn r#move(&self, from: Position, to: Position) -> Self {
        let mut new = *self;
        let piece = new[from].take().expect("Board::r#move precondition");
        let captured = new[to].replace(piece);
        // en passant
        if piece.piece == PieceType::Pawn && from.file() != to.file() && captured.is_none() {
            let captured_position = Position::new(from.rank(), to.file());
            let captured = new[captured_position].take();
            debug_assert_eq!(
                captured,
                Some(Piece {
                    color: !piece.color,
                    piece: PieceType::Pawn
                })
            );
        }
        // castling
        if piece.piece == PieceType::King && from.file().abs_diff(to.file()) == 2 {
            let (rook_from, rook_to) = if to.file() < from.file() {
                // queenside
                (to.left().left(), to.right())
            } else {
                // kingside
                (to.right(), to.left())
            };
            let (rook_from, rook_to) = (
                rook_from.expect("Board::move castling precondition"),
                rook_to.expect("Board::move castling precondition"),
            );
            let rook = new[rook_from].take();
            debug_assert_eq!(
                rook.expect("Board::move castling precondition"),
                Piece {
                    color: piece.color,
                    piece: PieceType::Rook
                }
            );
            new[rook_to] = rook;
        }
        new
    }

    pub fn promote(&self, from: Position, to: Position, piece_type: PieceType) -> Self {
        let mut new = self.clone();
        let piece = new[from].take().expect("Board::promote precondition");
        debug_assert_eq!(piece.piece, PieceType::Pawn);
        let piece = Piece {
            color: piece.color,
            piece: piece_type,
        };
        new[to] = Some(piece);
        new
    }

    pub fn position_of(&self, piece: Piece) -> Option<Position> {
        self.0
            .iter()
            .enumerate()
            .find_map(|(i, &p)| p.is_some_and(|p| p == piece).then(|| Position(i as u8)))
    }

    pub fn is_vacant(&self, position: Position) -> bool {
        self.get(position).is_none()
    }
}
