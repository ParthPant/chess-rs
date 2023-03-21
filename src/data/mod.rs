pub mod bitboard;
mod fen;
mod moves;
pub mod piece;
mod square;

use fen::Fen;
use moves::CastleType;
use std::str::FromStr;
use strum::IntoEnumIterator;

pub use bitboard::BitBoard;
pub use moves::{Move, MoveCommit, MoveList, MoveType};
pub use piece::{BoardPiece, Color};
pub use square::Square;

pub type BoardMap = [BitBoard; 12];

#[derive(Debug, Clone)]
pub struct BoardConfig {
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
        Fen::make_config_from_str("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
    }
}

impl BoardConfig {
    pub fn apply_move(&mut self, m: &Move) {
        log::info!("{:?}", m);
        let p = self.get_at_sq(m.from).unwrap();
        let pcolor = p.get_color();

        // prevent from moving when its not their turn
        if pcolor != self.active_color {
            return;
        }

        use MoveType::*;
        match m.move_type {
            Normal => self.apply_normal(m.from, m.to),
            DoublePush => self.apply_double_push(m.from, m.to, p),
            EnPassant => self.apply_en_passant(m.from, m.to, p),
            Castle(castle_type) => self.apply_castle(p, castle_type),
            Promotion(prom) => {
                if let Some(prom) = prom {
                    self.apply_promotion(m.from, m.to, prom);
                } else {
                    log::error!("Promotion Move has no promotion piece assigned to it");
                    panic!();
                }
            }
        }

        // en passant state update
        if m.move_type != DoublePush {
            self.en_passant_target = None;
        }
        // castling state update
        if p == BoardPiece::WhiteRook {
            if m.from == Square::A1 {
                self.can_white_castle_queenside = false;
            } else if m.from == Square::H1 {
                self.can_white_castle_kingside = false;
            }
        } else if p == BoardPiece::BlackRook {
            if m.from == Square::A8 {
                self.can_black_castle_queenside = false;
            } else if m.from == Square::H8 {
                self.can_black_castle_kingside = false;
            }
        } else if p == BoardPiece::WhiteKing {
            self.can_white_castle_kingside = false;
            self.can_white_castle_queenside = false;
        } else if p == BoardPiece::BlackKing {
            self.can_black_castle_kingside = false;
            self.can_black_castle_queenside = false;
        }

        if pcolor == Color::Black {
            self.fullmove_number += 1;
        }
        self.halfmove_clock += 1;
        self.toggle_active_color();
        self.fen_str = Fen::make_fen_from_config(self);
    }

    fn apply_normal(&mut self, from: Square, to: Square) {
        self.remove_piece(to);
        self.move_piece(from, to);
    }

    fn apply_double_push(&mut self, from: Square, to: Square, p: BoardPiece) {
        let pcolor = p.get_color();
        self.remove_piece(to);
        self.move_piece(from, to);
        if pcolor == Color::White {
            self.en_passant_target = Some(Square::try_from(to as usize - 8).unwrap());
        } else {
            self.en_passant_target = Some(Square::try_from(to as usize + 8).unwrap());
        }
    }

    fn apply_en_passant(&mut self, from: Square, to: Square, p: BoardPiece) {
        let pcolor = p.get_color();
        self.remove_piece(to);
        self.move_piece(from, to);
        if pcolor == Color::White {
            self.remove_piece(Square::try_from(to as usize - 8).unwrap());
        } else {
            self.remove_piece(Square::try_from(to as usize + 8).unwrap());
        }
    }

    fn apply_castle(&mut self, p: BoardPiece, castle_type: CastleType) {
        let pcolor = p.get_color();
        match castle_type {
            CastleType::KingSide => {
                if pcolor == Color::White {
                    self.move_piece(Square::E1, Square::G1);
                    self.move_piece(Square::H1, Square::F1);
                }
                if pcolor == Color::Black {
                    self.move_piece(Square::E8, Square::G8);
                    self.move_piece(Square::H8, Square::F8);
                }
            }
            CastleType::QueenSide => {
                if pcolor == Color::White {
                    self.move_piece(Square::E1, Square::C1);
                    self.move_piece(Square::A1, Square::D1);
                }
                if pcolor == Color::Black {
                    self.move_piece(Square::E8, Square::C8);
                    self.move_piece(Square::A8, Square::D8);
                }
            }
        }

        match pcolor {
            Color::White => {
                self.can_white_castle_kingside = false;
                self.can_white_castle_queenside = false;
            }
            Color::Black => {
                self.can_black_castle_kingside = false;
                self.can_black_castle_queenside = false;
            }
        }
    }

    fn apply_promotion(&mut self, from: Square, to: Square, prom: BoardPiece) {
        self.remove_piece(from);
        self.add_piece(prom, to);
    }

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

    pub fn get_at_sq(&self, sq: Square) -> Option<BoardPiece> {
        for piece in BoardPiece::iter() {
            if self.bitboards[piece as usize].is_set(sq) {
                return Some(piece);
            }
        }
        None
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
            return Some(self.bitboards[p as usize]);
        }
        None
    }

    pub fn get_piece_occupancy(&self, p: BoardPiece) -> BitBoard {
        self.bitboards[p as usize]
    }

    pub fn all_occupancy(&self) -> BitBoard {
        let mut ret = BitBoard::from(0);
        for bb in self.bitboards.iter() {
            ret |= *bb;
        }
        ret
    }

    pub fn white_occupancy(&self) -> BitBoard {
        let mut ret = BitBoard::from(0);
        use BoardPiece::*;
        ret |= self.bitboards[WhiteRook as usize]
            | self.bitboards[WhiteBishop as usize]
            | self.bitboards[WhiteKnight as usize]
            | self.bitboards[WhiteKing as usize]
            | self.bitboards[WhiteQueen as usize]
            | self.bitboards[WhitePawn as usize];
        ret
    }

    pub fn black_occupancy(&self) -> BitBoard {
        let mut ret = BitBoard::from(0);
        use BoardPiece::*;
        ret |= self.bitboards[BlackRook as usize]
            | self.bitboards[BlackBishop as usize]
            | self.bitboards[BlackKnight as usize]
            | self.bitboards[BlackKing as usize]
            | self.bitboards[BlackQueen as usize]
            | self.bitboards[BlackPawn as usize];
        ret
    }

    fn move_piece(&mut self, from: Square, to: Square) {
        let p = self.get_at_sq(from).unwrap();
        self.remove_piece(from);
        self.add_piece(p, to);
    }

    fn remove_piece(&mut self, from: Square) {
        if let Some(p) = self.get_at_sq(from) {
            self.remove_from_bitboard(p, from)
        }
    }

    fn add_piece(&mut self, p: BoardPiece, to: Square) {
        self.add_to_bitboard(p, to)
    }

    fn toggle_active_color(&mut self) {
        self.active_color = match self.active_color {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }

    fn update_bitboard(&mut self, p: BoardPiece, prev: Square, new: Square) {
        self.bitboards[p as usize].make_move(prev, new);
    }

    fn remove_from_bitboard(&mut self, p: BoardPiece, pos: Square) {
        self.bitboards[p as usize].unset(pos);
    }

    fn add_to_bitboard(&mut self, p: BoardPiece, pos: Square) {
        self.bitboards[p as usize].set(pos);
    }
}
