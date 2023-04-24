use std::fmt;
use std::str::FromStr;

use crate::data::{BoardMap, CastleFlags, GameState};
use crate::zobrist::hash;

use super::piece::{BoardPiece, Color};
use super::square::Square;
use super::BoardConfig;

pub struct Fen;

#[derive(Debug, Clone)]
pub enum FenError {
    InvalidColor(char),
    InvalidCastleFlag(char),
    InvalidEnPassantTarget(String),
    InvalidHalfMove(String),
    InvalidFullMove(String),
    ExtraFields,
}

impl fmt::Display for FenError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FenError::InvalidColor(c) => write!(f, "Invalid color: {}", c),
            FenError::InvalidCastleFlag(c) => write!(f, "Invalid castle flag: {}", c),
            FenError::InvalidEnPassantTarget(s) => write!(f, "Invalid en passant target: {}", s),
            FenError::InvalidHalfMove(s) => write!(f, "Invalid halfmove: {}", s),
            FenError::InvalidFullMove(s) => write!(f, "Invalid fullmove: {}", s),
            FenError::ExtraFields => write!(f, "Extra fields in FEN"),
        }
    }
}

impl Fen {
    pub fn make_config_from_str(s: &str) -> Result<BoardConfig, FenError> {
        Fen::make_config(s)
    }

    pub fn make_fen_from_config(c: &BoardConfig) -> String {
        let mut s = String::new();
        for y in (0..8).rev() {
            let mut empty = 0;
            for x in 0..8 {
                if let Some(p) = c.get_at_sq((x, y).try_into().unwrap()) {
                    if empty > 0 {
                        s.push_str(&empty.to_string());
                        empty = 0;
                    }
                    s.push_str(&p.to_string());
                } else {
                    empty = empty + 1;
                }
            }
            if empty > 0 {
                s.push_str(&empty.to_string());
            }
            if y > 0 {
                s.push('/');
            }
        }
        match c.get_active_color() {
            Color::White => s.push_str(" w "),
            Color::Black => s.push_str(" b "),
        }
        if c.get_can_white_castle_kingside() {
            s.push('K');
        }
        if c.get_can_white_castle_queenside() {
            s.push('Q');
        }
        if c.get_can_black_castle_kingside() {
            s.push('k');
        }
        if c.get_can_black_castle_queenside() {
            s.push('q');
        }
        s.push(' ');
        if let Some(pos) = c.get_en_passant_target() {
            s.push_str(&pos.to_string().to_lowercase());
        } else {
            s.push('-');
        }
        s.push(' ');
        s.push_str(&c.get_halfmove_clock().to_string());
        s.push(' ');
        s.push_str(&c.get_fullmove_number().to_string());
        s
    }

    fn get_piece_from_c(c: char) -> BoardPiece {
        if let Ok(p) = BoardPiece::from_str(&c.to_string()) {
            p
        } else {
            log::error!("Fen Error: {} is invalid piece", c);
            panic!();
        }
    }

    fn get_sq_from_code(s: &str) -> Square {
        let a = 'a'.to_ascii_lowercase() as usize;

        let mut it = s.chars();
        let c = it.next().unwrap();

        let x = if c.is_alphabetic() {
            c.to_ascii_lowercase() as usize - a
        } else {
            log::error!("Fen Error: {} is invalid square", s);
            panic!();
        };

        let n: String = it.collect();
        let y = n.parse::<usize>().unwrap() - 1;

        log::debug!("decode {} to {:?}", s, (x, y));

        (x, y).try_into().unwrap()
    }

    // TODO: Return Result with custom error type
    fn make_config(fen_str: &str) -> Result<BoardConfig, FenError> {
        log::trace!("Making BoardConfig...");
        let mut castle_flags = CastleFlags::default();
        let mut en_passant_target: Option<Square> = None;
        let mut halfmove_clock = 0;
        let mut fullmove_number = 0;
        let mut active_color = Color::White;
        let mut bitboards: BoardMap = Default::default();

        for (i, data) in fen_str.split_whitespace().enumerate() {
            log::trace!("Parcing Fen field {}, {}", i, data);
            match i {
                0 => {
                    for (i, rank) in data.split('/').enumerate() {
                        let mut x = 0;
                        for c in rank.chars() {
                            if c.is_digit(10) {
                                x = x + c.to_digit(10).unwrap();
                            } else {
                                log::debug!("Place {c} at {:?}", (7 - i, x));
                                bitboards[Fen::get_piece_from_c(c) as usize]
                                    .set(Square::try_from((x as usize, 7 - i)).unwrap());
                                x = x + 1;
                            }
                        }
                    }
                }
                1 => {
                    if data.len() > 1 {
                        log::error!("Fen Error: Active color field is wrong");
                    } else {
                        if let Some(c) = data.chars().next() {
                            match c {
                                'w' => {
                                    active_color = Color::White;
                                }
                                'b' => {
                                    active_color = Color::Black;
                                }
                                _ => {
                                    Err(FenError::InvalidColor(c))?;
                                }
                            }
                        }
                    }
                }
                2 => {
                    if data.len() == 1 && data.chars().next() == Some('-') {
                    } else {
                        let mut chars = data.chars();
                        while let Some(c) = chars.next() {
                            match c {
                                'k' => castle_flags.set_black_oo(),
                                'q' => castle_flags.set_black_ooo(),
                                'K' => castle_flags.set_white_oo(),
                                'Q' => castle_flags.set_white_ooo(),
                                _ => Err(FenError::InvalidCastleFlag(c))?,
                            }
                        }
                    }
                }
                3 => {
                    if data.len() == 1 && data.chars().next() == Some('-') {
                    } else {
                        en_passant_target = Some(Self::get_sq_from_code(data));
                    }
                }
                4 => {
                    if let Ok(n) = data.parse::<u8>() {
                        halfmove_clock = n;
                    } else {
                        Err(FenError::InvalidHalfMove(data.to_string()))?
                    }
                }
                5 => {
                    if let Ok(n) = data.parse::<u8>() {
                        fullmove_number = n;
                    } else {
                        Err(FenError::InvalidFullMove(data.to_string()))?
                    }
                }
                _ => {
                    Err(FenError::ExtraFields)?
                }
            };
        }

        log::trace!("Done..");
        let mut c = BoardConfig {
            active_color,
            en_passant_target,
            castle_flags,
            halfmove_clock,
            fullmove_number,
            bitboards,
            move_history: Default::default(),
            state: GameState::InPlay,
            hash: 0,
        };
        c.hash = hash(&c);
        Ok(c)
    }
}
