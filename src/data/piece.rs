use strum_macros::{Display, EnumString};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Display)]
pub enum Color {
    White,
    Black,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Display, EnumString)]
pub enum BoardPiece {
    #[strum(serialize="K")]
    WhiteKing,
    #[strum(serialize="R")]
    WhiteRook,
    #[strum(serialize="B")]
    WhiteBishop,
    #[strum(serialize="Q")]
    WhiteQueen,
    #[strum(serialize="N")]
    WhiteKnight,
    #[strum(serialize="P")]
    WhitePawn,

    #[strum(serialize="k")]
    BlackKing,
    #[strum(serialize="r")]
    BlackRook,
    #[strum(serialize="b")]
    BlackBishop,
    #[strum(serialize="q")]
    BlackQueen,
    #[strum(serialize="n")]
    BlackKnight,
    #[strum(serialize="p")]
    BlackPawn,
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
}
