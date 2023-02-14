use std::fmt;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Piece {
    King,
    Rook,
    Bishop,
    Queen,
    Knight,
    Pawn,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Color {
    White,
    Black,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum BoardPiece {
    White(Piece),
    Black(Piece),
}

impl fmt::Display for BoardPiece {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use BoardPiece::*;
        use Piece::*;

        let name = match self {
            White(p) => match p {
                King => "king_w",
                Rook => "rook_w",
                Bishop => "bishop_w",
                Queen => "queen_w",
                Knight => "knight_w",
                Pawn => "pawn_w",
            },
            Black(p) => match p {
                King => "king_b",
                Rook => "rook_b",
                Bishop => "bishop_b",
                Queen => "queen_b",
                Knight => "knight_b",
                Pawn => "pawn_b",
            },
        };

        write!(f, "{}", name)
    }
}
