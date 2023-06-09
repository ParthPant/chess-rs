use std::convert::{Into, TryFrom};
use strum_macros::{Display, EnumIter, EnumString};

macro_rules! make_enum {
    ($(#[$meta:meta])* $vis:vis enum $name:ident {
        $($(#[$vmeta:meta])* $vname:ident $(= $val:expr)?,)*
    }) => {
        $(#[$meta])*
        $vis enum $name {
            $($(#[$vmeta])* $vname $(= $val)?,)*
        }

        impl TryFrom<usize> for $name {
            type Error = SquareFromUsizeErr;

            fn try_from(v: usize) -> Result<Self, Self::Error> {
                match v {
                    $(x if x == $name::$vname as usize => Ok($name::$vname),)*
                    _ => Err(SquareFromUsizeErr(format!("failed to convert {:?} to Square", v))),
                }
            }
        }
    }
}

make_enum! {
    #[repr(u8)]
    #[derive(Ord, PartialOrd, Eq, PartialEq, Debug, Clone, Copy, EnumIter, EnumString, Display)]
    #[strum(ascii_case_insensitive, serialize_all = "lowercase")]
    pub enum Square {
        A1, B1, C1, D1, E1, F1, G1, H1,
        A2, B2, C2, D2, E2, F2, G2, H2,
        A3, B3, C3, D3, E3, F3, G3, H3,
        A4, B4, C4, D4, E4, F4, G4, H4,
        A5, B5, C5, D5, E5, F5, G5, H5,
        A6, B6, C6, D6, E6, F6, G6, H6,
        A7, B7, C7, D7, E7, F7, G7, H7,
        A8, B8, C8, D8, E8, F8, G8, H8,
    }
}

use Square::*;
#[rustfmt::skip]
const MIRROR: [Square; 64] =
[
    A8, B8, C8, D8, E8, F8, G8, H8,
    A7, B7, C7, D7, E7, F7, G7, H7,
    A6, B6, C6, D6, E6, F6, G6, H6,
    A5, B5, C5, D5, E5, F5, G5, H5,
    A4, B4, C4, D4, E4, F4, G4, H4,
    A3, B3, C3, D3, E3, F3, G3, H3,
    A2, B2, C2, D2, E2, F2, G2, H2,
    A1, B1, C1, D1, E1, F1, G1, H1,
];

impl TryFrom<(usize, usize)> for Square {
    type Error = SquareFromPairErr;

    fn try_from(pos: (usize, usize)) -> Result<Self, Self::Error> {
        match (pos.1 * 8 + pos.0).try_into() {
            Ok(sq) => Ok(sq),
            Err(err) => Err(SquareFromPairErr(format!(
                "failed to convert {:?} to Square: {}",
                pos, err.0
            ))),
        }
    }
}

impl Into<(usize, usize)> for Square {
    fn into(self) -> (usize, usize) {
        let s = self as usize;
        (s % 8, s / 8)
    }
}

impl Square {
    pub const fn mirror(&self) -> Self {
        MIRROR[*self as usize]
    }
}

#[derive(Debug)]
pub struct SquareFromUsizeErr(String);
#[derive(Debug)]
pub struct SquareFromPairErr(String);
