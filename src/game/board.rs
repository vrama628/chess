use std::collections::BTreeMap;

use crate::game::{
    piece::{Piece, PieceColor, PieceType},
    position::Position,
};

#[derive(Clone)]
pub struct Board(BTreeMap<Position, Piece>);

impl Board {
    pub fn new() -> Self {
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

    pub fn get(&self, position: Position) -> Option<&Piece> {
        self.0.get(&position)
    }

    pub fn iter(&self, color: PieceColor) -> impl Iterator<Item = (&Position, &Piece)> {
        self.0.iter().filter(move |(_, piece)| piece.color == color)
    }

    /// REQUIRES: there is a piece at `from`
    pub fn r#move(&self, from: Position, to: Position) -> Self {
        // TODO: use im
        let mut new = self.clone();
        let piece = new.0.remove(&from).expect("Board::r#move precondition");
        let captured = new.0.insert(to, piece);
        // en passant
        if piece.piece == PieceType::Pawn && from.file != to.file && captured.is_none() {
            let captured_position = Position {
                rank: from.rank,
                file: to.file,
            };
            let captured = new.0.remove(&captured_position);
            debug_assert_eq!(
                captured,
                Some(Piece {
                    color: !piece.color,
                    piece: PieceType::Pawn
                })
            );
        }
        // castling
        if piece.piece == PieceType::King && from.file.abs_diff(to.file) == 2 {
            let (rook_from, rook_to) = if to.file < from.file {
                // queenside
                (to.left().left(), to.right())
            } else {
                // kingside
                (to.right(), to.left())
            };
            let rook = new
                .0
                .remove(&rook_from)
                .expect("Board::r#move castling precondition");
            debug_assert_eq!(
                rook,
                Piece {
                    color: piece.color,
                    piece: PieceType::Rook
                }
            );
            new.0.insert(rook_to, rook);
        }
        new
    }

    pub fn promote(&self, from: Position, to: Position, piece_type: PieceType) -> Self {
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

    pub fn position_of(&self, piece: Piece) -> Option<Position> {
        self.0
            .iter()
            .find_map(|(&position, &p)| (p == piece).then_some(position))
    }

    pub fn is_vacant(&self, position: Position) -> bool {
        self.get(position).is_none()
    }
}
