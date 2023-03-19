pub mod bitboard;
mod fen;
mod moves;
pub mod piece;
mod square;

use bitboard::BitBoard;
use fen::Fen;
use piece::{BoardPiece, Color};
use std::{collections::HashMap, str::FromStr};
use strum::IntoEnumIterator;

pub use moves::{Move, MoveList};
pub use square::Square;

pub type BoardMatrix = [[Option<BoardPiece>; 8]; 8];
pub type BoardMap = HashMap<BoardPiece, BitBoard>;

#[derive(Debug, Clone)]
pub struct BoardConfig {
    board_mat: BoardMatrix,
    fen_str: String,
    active_color: Color,
    en_passant_target: Option<Square>,
    can_white_castle_queenside: bool,
    can_white_castle_kingside: bool,
    can_black_castle_queenside: bool,
    can_black_castle_kingside: bool,
    halfmove_clock: u32,
    fullmove_number: u32,
    pub bitboards: BoardMap,
}

impl Default for BoardConfig {
    fn default() -> Self {
        let mut c =
            Fen::make_config_from_str("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        c.make_bitboards();
        c
    }
}

impl BoardConfig {
    pub fn reset(&mut self) {
        *self = BoardConfig::default();
    }

    pub fn from_fen_str(s: &str) -> Self {
        let mut c = Fen::make_config_from_str(s);
        c.make_bitboards();
        c
    }

    pub fn load_fen(&mut self, s: &str) {
        *self = Fen::make_config_from_str(s);
        self.make_bitboards();
    }

    pub fn get_fen(&self) -> &str {
        &self.fen_str
    }

    pub fn get_at_sq(&self, sq: Square) -> Option<BoardPiece> {
        let (x, y) = sq.into();
        self.board_mat[y][x]
    }

    pub fn make_move(&mut self, m: &Move) {
        let prev = m.from;
        let new = m.to;
        if prev != new {
            let previ: (usize, usize) = prev.into();
            let newi: (usize, usize) = new.into();
            let p = self.board_mat[previ.1][previ.0].unwrap();
            let pcolor = p.get_color();

            // prevent from moving when its not their turn
            if pcolor != self.active_color {
                return;
            }
            if let Some(cap) = self.board_mat[newi.1][newi.0] {
                // prevent from moving to a square with piece of same color
                if cap.get_color() == pcolor {
                    return;
                }
                // capture piece
                // TODO: Handle Captures
                self.remove_piece(m.to);
            }

            if m.ep {
                self.en_passant_target = None;
                self.move_piece(m.from, m.to);
                if pcolor == Color::White {
                    self.remove_piece(Square::try_from(m.to as usize - 8).unwrap());
                } else {
                    self.remove_piece(Square::try_from(m.to as usize + 8).unwrap());
                }
            } else if m.castle_oo {
                if pcolor == Color::White {
                    self.move_piece(Square::E1, Square::G1);
                    self.move_piece(Square::H1, Square::F1);
                }
                if pcolor == Color::Black {
                    self.move_piece(Square::E8, Square::G8);
                    self.move_piece(Square::H8, Square::F8);
                }
            } else if m.castle_ooo {
                if pcolor == Color::White {
                    self.move_piece(Square::E1, Square::C1);
                    self.move_piece(Square::A1, Square::D1);
                }
                if pcolor == Color::Black {
                    self.move_piece(Square::E8, Square::C8);
                    self.move_piece(Square::A8, Square::D8);
                }
            } else if let Some(promotion) = m.promotion {
                self.remove_piece(m.from);
                self.add_piece(promotion, m.to);
            } else {
                self.move_piece(m.from, m.to);
            }
            self.toggle_active_color();
            if m.castle_oo || m.castle_oo {
                if pcolor == Color::White {
                    self.can_white_castle_kingside = false;
                    self.can_white_castle_queenside = false;
                }
                if pcolor == Color::Black {
                    self.can_black_castle_kingside = false;
                    self.can_black_castle_queenside = false;
                }
            }
            if p == BoardPiece::WhiteRook {
                if m.from == Square::A1 {
                    self.can_white_castle_queenside = false;
                } else if m.from == Square::H1 {
                    self.can_white_castle_kingside = false;
                }
            }
            if p == BoardPiece::BlackRook {
                if m.from == Square::A8 {
                    self.can_black_castle_queenside = false;
                } else if m.from == Square::H8 {
                    self.can_black_castle_kingside = false;
                }
            }
            if p == BoardPiece::WhiteKing {
                self.can_white_castle_kingside = false;
                self.can_white_castle_queenside = false;
            }
            if p == BoardPiece::BlackKing {
                self.can_black_castle_kingside = false;
                self.can_black_castle_queenside = false;
            }
            if m.double_push {
                if pcolor == Color::White {
                    self.en_passant_target = Some(Square::try_from(m.to as usize - 8).unwrap());
                } else {
                    self.en_passant_target = Some(Square::try_from(m.to as usize + 8).unwrap());
                }
            } else {
                self.en_passant_target = None;
            }
            if pcolor == Color::Black {
                self.fullmove_number += 1;
            }
            self.halfmove_clock += 1;
            self.fen_str = Fen::make_fen_from_config(&self);
            log::info!("Move {:?}", m);
            log::info!("Fen: {}", self.fen_str);
        }
    }

    pub fn get_active_color(&self) -> Color {
        self.active_color
    }

    pub fn get_can_white_castle_queenside(&self) -> bool {
        self.can_white_castle_queenside
    }

    pub fn get_can_white_castle_kingside(&self) -> bool {
        self.can_white_castle_kingside
    }

    pub fn get_can_black_castle_queenside(&self) -> bool {
        self.can_black_castle_queenside
    }

    pub fn get_can_black_castle_kingside(&self) -> bool {
        self.can_black_castle_kingside
    }

    pub fn get_en_passant_target(&self) -> Option<Square> {
        self.en_passant_target
    }

    pub fn get_halfmove_clock(&self) -> u32 {
        self.halfmove_clock
    }

    pub fn get_fullmove_number(&self) -> u32 {
        self.fullmove_number
    }

    pub fn get_bit_board(&self, c: char) -> Option<BitBoard> {
        if let Ok(p) = BoardPiece::from_str(&c.to_string()) {
            if let Some(b) = self.bitboards.get(&p) {
                return Some(*b);
            }
        }
        None
    }

    pub fn get_piece_occupancy(&self, p: BoardPiece) -> BitBoard {
        *self.bitboards.get(&p).unwrap()
    }

    pub fn all_occupancy(&self) -> BitBoard {
        let mut ret = BitBoard::from(0);
        for (_p, bb) in self.bitboards.iter() {
            ret |= *bb;
        }
        ret
    }

    pub fn white_occupancy(&self) -> BitBoard {
        let mut ret = BitBoard::from(0);
        use BoardPiece::*;
        ret |= self.bitboards[&WhiteRook]
            | self.bitboards[&WhiteBishop]
            | self.bitboards[&WhiteKnight]
            | self.bitboards[&WhiteKing]
            | self.bitboards[&WhiteQueen]
            | self.bitboards[&WhitePawn];
        ret
    }

    pub fn black_occupancy(&self) -> BitBoard {
        let mut ret = BitBoard::from(0);
        use BoardPiece::*;
        ret |= self.bitboards[&BlackRook]
            | self.bitboards[&BlackBishop]
            | self.bitboards[&BlackKnight]
            | self.bitboards[&BlackKing]
            | self.bitboards[&BlackQueen]
            | self.bitboards[&BlackPawn];
        ret
    }

    fn move_piece(&mut self, from: Square, to: Square) {
        let previ: (usize, usize) = from.into();
        let p = self.board_mat[previ.1][previ.0].unwrap();
        self.remove_piece(from);
        self.add_piece(p, to)
    }

    fn remove_piece(&mut self, from: Square) {
        let loc: (usize, usize) = from.into();
        let p = self.board_mat[loc.1][loc.0].unwrap();
        self.board_mat[loc.1][loc.0] = None;
        self.remove_from_bitboard(p, from)
    }

    fn add_piece(&mut self, p: BoardPiece, to: Square) {
        let loc: (usize, usize) = to.into();
        self.board_mat[loc.1][loc.0] = Some(p);
        self.add_to_bitboard(p, to)
    }

    fn toggle_active_color(&mut self) {
        self.active_color = match self.active_color {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }

    fn make_bitboards(&mut self) {
        self.bitboards.clear();
        for p in BoardPiece::iter() {
            self.bitboards.entry(p).or_insert(BitBoard::from(0));
        }
        for y in 0..8 {
            for x in 0..8 {
                if let Some(p) = self.board_mat[y][x] {
                    self.bitboards.entry(p).and_modify(|b| {
                        b.set((x, y).try_into().unwrap());
                    });
                    log::debug!("add {} to bit {}", p, 8 * y + x);
                }
            }
        }
    }

    fn update_bitboard(&mut self, p: BoardPiece, prev: Square, new: Square) {
        self.bitboards.entry(p).and_modify(|b| {
            b.make_move(prev, new);
        });
    }

    fn remove_from_bitboard(&mut self, p: BoardPiece, pos: Square) {
        self.bitboards.entry(p).and_modify(|b| {
            b.unset(pos);
        });
    }

    fn add_to_bitboard(&mut self, p: BoardPiece, pos: Square) {
        self.bitboards.entry(p).and_modify(|b| {
            b.set(pos);
        });
    }
}
