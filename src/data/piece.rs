use std::fmt;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Color {
    White,
    Black,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum BoardPiece {
    WhiteKing,
    WhiteRook,
    WhiteBishop,
    WhiteQueen,
    WhiteKnight,
    WhitePawn,

    BlackKing,
    BlackRook,
    BlackBishop,
    BlackQueen,
    BlackKnight,
    BlackPawn,
}

impl fmt::Display for BoardPiece {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use BoardPiece::*;

        let name = match self {
            WhiteKing => "K",
            WhiteRook => "R",
            WhiteBishop => "B",
            WhiteQueen => "Q",
            WhiteKnight => "N",
            WhitePawn => "P",

            BlackKing => "k",
            BlackRook => "r",
            BlackBishop => "b",
            BlackQueen => "q",
            BlackKnight => "n",
            BlackPawn => "p",
        };

        write!(f, "{}", name)
    }
}
