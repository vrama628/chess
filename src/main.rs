use ratatui::{crossterm::event, layout::Flex, prelude::*, widgets::Block};
use std::collections::BTreeMap;

enum PieceType {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

impl PieceType {
    fn render<'a>(&self, block: Block<'a>) -> Block<'a> {
        let content = match self {
            Self::Pawn => "♟",
            Self::Knight => "♞",
            Self::Bishop => "♝",
            Self::Rook => "♜",
            Self::Queen => "♛",
            Self::King => "♚",
        };
        block.title(content)
    }
}

enum PieceColor {
    White,
    Black,
}

impl PieceColor {
    fn render<'a>(&self, block: Block<'a>) -> Block<'a> {
        let fg = match self {
            Self::White => Color::White,
            Self::Black => Color::Black,
        };
        block.fg(fg)
    }
}

struct Piece {
    color: PieceColor,
    piece: PieceType,
}

impl Piece {
    fn render<'a>(&self, block: Block<'a>) -> Block<'a> {
        self.color.render(self.piece.render(block))
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct Position {
    rank: usize,
    file: usize,
}

impl Position {
    fn square(&self) -> Block<'static> {
        let bg = if (self.rank + self.file) % 2 == 0 {
            Color::DarkGray
        } else {
            Color::Gray
        };
        Block::new().bg(bg)
    }
}

struct Board(BTreeMap<Position, Piece>);

impl Board {
    fn new() -> Self {
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
        let black_pieces = PIECE_TYPES.into_iter().enumerate().map(|(file, piece)| {
            (
                Position { rank: 7, file },
                Piece {
                    color: PieceColor::Black,
                    piece,
                },
            )
        });
        let black_pawns = (0..8).map(|file| {
            (
                Position { rank: 6, file },
                Piece {
                    color: PieceColor::Black,
                    piece: PieceType::Pawn,
                },
            )
        });
        let white_pawns = (0..8).map(|file| {
            (
                Position { rank: 1, file },
                Piece {
                    color: PieceColor::White,
                    piece: PieceType::Pawn,
                },
            )
        });
        let white_pieces = PIECE_TYPES.into_iter().enumerate().map(|(file, piece)| {
            (
                Position { rank: 0, file },
                Piece {
                    color: PieceColor::White,
                    piece,
                },
            )
        });
        Self(
            black_pieces
                .chain(black_pawns)
                .chain(white_pawns)
                .chain(white_pieces)
                .collect(),
        )
    }
}

impl Widget for &Board {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let [area] = Layout::horizontal([8 * 2]).flex(Flex::Center).areas(area);
        let [area] = Layout::vertical([8]).flex(Flex::Center).areas(area);
        let ranks = Layout::vertical([Constraint::Fill(1); 8]).split(area);
        for (rank, rect) in ranks.iter().copied().rev().enumerate() {
            let files = Layout::horizontal([Constraint::Fill(1); 8]).split(rect);
            for (file, rect) in files.iter().copied().enumerate() {
                let position = Position { rank, file };
                let mut span = position.square();
                if let Some(piece) = self.0.get(&position) {
                    span = piece.render(span);
                }
                span.render(rect, buf)
            }
        }
    }
}

fn main() -> Result<(), std::io::Error> {
    let board = Board::new();
    let mut terminal = ratatui::init();
    terminal.draw(|frame| frame.render_widget(&board, frame.area()))?;
    event::read()?;
    ratatui::restore();
    Ok(())
}
