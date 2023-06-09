use std::ops::Not;
use strum_macros::{Display, EnumIter, EnumString};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Display)]
pub enum Color {
    White,
    Black,
}

impl Not for Color {
    type Output = Self;

    fn not(self) -> Self {
        use Color::*;
        match self {
            White => Black,
            Black => White,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Display, EnumString, EnumIter)]
pub enum BoardPiece {
    #[strum(serialize = "P")]
    WhitePawn,
    #[strum(serialize = "N")]
    WhiteKnight,
    #[strum(serialize = "B")]
    WhiteBishop,
    #[strum(serialize = "R")]
    WhiteRook,
    #[strum(serialize = "Q")]
    WhiteQueen,
    #[strum(serialize = "K")]
    WhiteKing,

    #[strum(serialize = "p")]
    BlackPawn,
    #[strum(serialize = "n")]
    BlackKnight,
    #[strum(serialize = "b")]
    BlackBishop,
    #[strum(serialize = "r")]
    BlackRook,
    #[strum(serialize = "q")]
    BlackQueen,
    #[strum(serialize = "k")]
    BlackKing,
}

impl BoardPiece {
    pub fn get_color(&self) -> Color {
        use BoardPiece::*;
        match self {
            WhiteKing | WhiteRook | WhiteBishop | WhiteQueen | WhiteKnight | WhitePawn => {
                Color::White
            }

            BlackKing | BlackRook | BlackBishop | BlackQueen | BlackKnight | BlackPawn => {
                Color::Black
            }
        }
    }

    pub fn utf_str(&self) -> &'static str {
        use BoardPiece::*;
        match self {
            WhitePawn => "♙",
            WhiteKnight => "♘",
            WhiteBishop => "♗",
            WhiteRook => "♖",
            WhiteQueen => "♕",
            WhiteKing => "♔",
            BlackPawn => "♟",
            BlackKnight => "♞",
            BlackBishop => "♝",
            BlackRook => "♜",
            BlackQueen => "♛",
            BlackKing => "♚",
        }
    }
}

pub const W_PIECES: [BoardPiece; 6] = {
    use BoardPiece::*;
    [
        WhiteKing,
        WhitePawn,
        WhiteRook,
        WhiteQueen,
        WhiteBishop,
        WhiteKnight,
    ]
};

pub const B_PIECES: [BoardPiece; 6] = {
    use BoardPiece::*;
    [
        BlackKing,
        BlackPawn,
        BlackRook,
        BlackQueen,
        BlackBishop,
        BlackKnight,
    ]
};
