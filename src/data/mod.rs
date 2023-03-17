pub mod bitboard;
mod fen;
pub mod piece;

use std::collections::HashMap;

use bitboard::BitBoard;
use fen::{Fen, PIECES_CHARS};
use piece::{BoardPiece, Color};

pub type BoardMatrix = [[Option<BoardPiece>; 8]; 8];
pub type BoardMap = HashMap<BoardPiece, BitBoard>;

#[derive(Debug, Clone)]
pub struct BoardConfig {
    board_mat: BoardMatrix,
    fen_str: String,
    active_color: Color,
    en_passant_target: Option<(usize, usize)>,
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
        Fen::make_config_from_str(s)
    }

    pub fn load_fen(&mut self, s: &str) {
        *self = Fen::make_config_from_str(s);
    }

    pub fn get_fen(&self) -> &str {
        &self.fen_str
    }

    pub fn get_at_xy(&self, x: usize, y: usize) -> Option<BoardPiece> {
        self.board_mat[y][x]
    }

    pub fn move_xy_to_xy(&mut self, prev: (usize, usize), new: (usize, usize)) {
        if prev != new {
            let p = self.board_mat[prev.1][prev.0].unwrap();
            let pcolor = p.get_color();
            // prevent from moving when its not their turn
            if pcolor != self.active_color {
                return;
            }
            if let Some(to) = self.board_mat[new.1][new.0] {
                // prevent from moving to a square with piece of same color
                if to.get_color() == pcolor {
                    return;
                }
                // capture piece
                // TODO: Handle Captures
                self.remove_from_bitboard(to, new);
            }
            self.board_mat[new.1][new.0] = self.board_mat[prev.1][prev.0];
            self.board_mat[prev.1][prev.0] = None;
            self.toggle_active_color();
            if pcolor == Color::Black {
                self.fullmove_number += 1;
            }
            self.halfmove_clock += 1;
            self.fen_str = Fen::make_fen_from_config(&self);
            self.update_bitboard(p, prev, new);
            log::info!("Move {:?} to {:?}", prev, new);
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

    pub fn get_en_passant_target(&self) -> Option<(usize, usize)> {
        self.en_passant_target
    }

    pub fn get_halfmove_clock(&self) -> u32 {
        self.halfmove_clock
    }

    pub fn get_fullmove_number(&self) -> u32 {
        self.fullmove_number
    }

    pub fn get_bit_board(&self, c: char) -> Option<BitBoard> {
        if let Some(p) = PIECES_CHARS.get(&c) {
            if let Some(b) = self.bitboards.get(p) {
                return Some(*b);
            }
        }
        None
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

    fn toggle_active_color(&mut self) {
        self.active_color = match self.active_color {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }

    fn make_bitboards(&mut self) {
        for y in 0..8 {
            for x in 0..8 {
                if let Some(p) = self.board_mat[y][x] {
                    let b = self.bitboards.entry(p).or_default();
                    log::debug!("add {} to bit {}", p, 8 * y + x);
                    b.add(x, y);
                }
            }
        }
    }

    fn update_bitboard(&mut self, p: BoardPiece, prev: (usize, usize), new: (usize, usize)) {
        self.bitboards.entry(p).and_modify(|b| {
            b.move_xy_to_xy(prev, new);
        });
    }

    fn remove_from_bitboard(&mut self, p: BoardPiece, pos: (usize, usize)) {
        self.bitboards.entry(p).and_modify(|b| {
            b.unset(pos.1 * 8 + pos.0);
        });
    }
}
