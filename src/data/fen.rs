use std::collections::HashMap;

use super::piece::{BoardPiece, BoardPiece::*, Color};
use super::BoardConfig;
use super::BoardMatrix;
use phf::phf_map;

pub static PIECES_CHARS: phf::Map<char, BoardPiece> = phf_map! {
    'k' => BlackKing,
    'K' => WhiteKing,

    'r' => BlackRook,
    'R' => WhiteRook,

    'b' => BlackBishop,
    'B' => WhiteBishop,

    'q' => BlackQueen,
    'Q' => WhiteQueen,

    'n' => BlackKnight,
    'N' => WhiteKnight,

    'p' => BlackPawn,
    'P' => WhitePawn,
};

pub struct Fen {}

impl Fen {
    pub fn make_config_from_str(s: &str) -> BoardConfig {
        Fen::make_config(s)
    }

    pub fn make_fen_from_config(c: &BoardConfig) -> String {
        let mut s = String::new();
        for y in (0..8).rev() {
            let mut empty = 0;
            for x in 0..8 {
                if let Some(p) = c.get_at_xy(x, y) {
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
            s.push('/');
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
            s.push_str(&Fen::get_code_from_mat_coords(pos.0, pos.1));
        } else {
            s.push('-');
        }
        s.push(' ');
        s.push_str(&c.get_halfmove_clock().to_string());
        s.push(' ');
        s.push_str(&c.get_fullmove_number().to_string());
        s
    }

    fn get_code_from_mat_coords(x: usize, y: usize) -> String {
        let c = 'a'.to_ascii_lowercase() as u8 + x as u8;
        let y = (y + 1) as u8;
        std::str::from_utf8(&[c, y]).unwrap().to_string()
    }

    // fn get_c_from_piece(p: BoardPiece) -> char {
    //     match p {
    //         BlackKing => 'k',
    //         WhiteKing => 'K',
    //
    //         BlackRook => 'r',
    //         WhiteRook => 'R',
    //
    //         BlackBishop => 'b',
    //         WhiteBishop => 'B',
    //
    //         BlackQueen => 'q',
    //         WhiteQueen => 'Q',
    //
    //         BlackKnight => 'n',
    //         WhiteKnight => 'N',
    //
    //         BlackPawn => 'p',
    //         WhitePawn => 'P',
    //     }
    // }

    fn get_piece_from_c(c: char) -> BoardPiece {
        if let Some(p) = PIECES_CHARS.get(&c).cloned() {
            p
        } else {
            log::error!("Fen Error: {} is invalid piece", c);
            panic!();
        }
    }

    fn get_matrix_coords_from_code(s: &str) -> (usize, usize) {
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

        (x, y)
    }

    // TODO: Return Result with custom error type
    fn make_config(fen_str: &str) -> BoardConfig {
        log::trace!("Making BoardConfig...");
        let mut board_mat = BoardMatrix::default();
        let mut can_white_castle_queenside = false;
        let mut can_white_castle_kingside = false;
        let mut can_black_castle_queenside = false;
        let mut can_black_castle_kingside = false;
        let mut en_passant_target: Option<(usize, usize)> = None;
        let mut halfmove_clock = 0;
        let mut fullmove_number = 0;
        let mut active_color = Color::White;

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
                                board_mat[7 - i][x as usize] = Some(Fen::get_piece_from_c(c));
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
                                    log::error!("Fen Error: {} is invalid color", c);
                                    panic!();
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
                                'k' => can_black_castle_kingside = true,
                                'q' => can_black_castle_queenside = true,
                                'K' => can_white_castle_kingside = true,
                                'Q' => can_white_castle_queenside = true,
                                _ => {
                                    log::error!("Fen Error: {} is invalid", c);
                                    panic!();
                                }
                            }
                        }
                    }
                }
                3 => {
                    if data.len() == 1 && data.chars().next() == Some('-') {
                    } else {
                        en_passant_target = Some(Self::get_matrix_coords_from_code(data));
                    }
                }
                4 => {
                    if let Ok(n) = data.parse::<u32>() {
                        halfmove_clock = n;
                    } else {
                        log::error!("Fen Error: {} is invalid halfmove", data);
                        panic!();
                    }
                }
                5 => {
                    if let Ok(n) = data.parse::<u32>() {
                        fullmove_number = n;
                    } else {
                        log::error!("Fen Error: {} is invalid fullmove", data);
                        panic!();
                    }
                }
                _ => {
                    log::error!("Fen Error: Extra Fields");
                    panic!();
                }
            };
        }

        log::trace!("Done..");
        BoardConfig {
            board_mat,
            fen_str: fen_str.to_string(),
            active_color,
            en_passant_target,
            can_white_castle_kingside,
            can_white_castle_queenside,
            can_black_castle_kingside,
            can_black_castle_queenside,
            halfmove_clock,
            fullmove_number,
            bitboards: HashMap::default(),
        }
    }
}
