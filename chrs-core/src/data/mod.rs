pub mod bitboard;
mod fen;
mod moves;
pub mod piece;
mod square;

use crate::zobrist::{hash, update_castle, update_ep, update_side};
use crate::{generator::MoveGenerator, zobrist::update_piece};
use fen::Fen;
use moves::CastleType;
use std::str::FromStr;
use strum::IntoEnumIterator;

pub use bitboard::BitBoard;
pub use moves::{Move, MoveCommit, MoveHistory, MoveList, MoveType};
pub use piece::{BoardPiece, Color, B_PIECES, W_PIECES};
pub use square::Square;

pub type BoardMap = [BitBoard; 12];

#[derive(Debug, Clone)]
pub struct BoardConfig {
    active_color: Color,
    en_passant_target: Option<Square>,
    castle_flags: CastleFlags,
    halfmove_clock: u32,
    fullmove_number: u32,
    pub bitboards: BoardMap,
    move_history: MoveHistory,
    hash: u64,
}

impl Default for BoardConfig {
    fn default() -> Self {
        Fen::make_config_from_str("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
    }
}

impl BoardConfig {
    pub fn get_hash(&self) -> u64 {
        self.hash
    }

    pub fn print_board(&self) {
        println!("{}", self.to_string());
    }

    fn set_ep_target(&mut self, t: Square) {
        if let Some(t) = self.en_passant_target {
            self.hash = update_ep(t, self.hash)
        }
        self.en_passant_target = Some(t);
        self.hash = update_ep(t, self.hash)
    }

    fn clear_ep_target(&mut self) {
        if let Some(t) = self.en_passant_target {
            self.hash = update_ep(t, self.hash)
        }
        self.en_passant_target = None;
    }

    pub fn to_string(&self) -> String {
        let mut s = String::new();
        for y in (0..8).rev() {
            s = format!("{}{}", s, y + 1);
            for x in 0..8 {
                let sq = Square::try_from((x, y)).unwrap();
                if let Some(p) = self.get_at_sq(sq) {
                    s = format!("{}  {}", s, p.utf_str());
                } else {
                    s = format!("{}  .", s);
                }
            }
            s = format!("{}\n", s);
        }
        s = format!("{}   a  b  c  d  e  f  g  h", s);
        s = format!("{}\nHash: {:x}", s, hash(self));
        s
    }

    pub fn get_last_commit(&self) -> Option<MoveCommit> {
        self.move_history.get_last()
    }

    pub fn apply_move(&mut self, m: Move) {
        if let Some(commit) = self.make_move(m) {
            log::info!("{:?}", commit);
            self.move_history.push(commit);
        }
    }

    pub fn is_king_in_check(&self, gen: &MoveGenerator, side: Color) -> bool {
        let sq = match side {
            Color::White => self.bitboards[BoardPiece::WhiteKing as usize].peek(),
            Color::Black => self.bitboards[BoardPiece::BlackKing as usize].peek(),
        };
        if let Some(sq) = sq {
            return gen.is_sq_attacked(sq, !side, &self);
        }
        false
    }

    pub fn make_move(&mut self, m: Move) -> Option<MoveCommit> {
        let p = self.get_at_sq(m.from).unwrap();
        let pcolor = p.get_color();

        // prevent from moving when its not their turn
        if pcolor != self.active_color {
            return None;
        }

        let prev_ep_target = self.en_passant_target;
        let prev_castle_flags = self.castle_flags;

        use MoveType::*;
        let cap = match m.move_type {
            Normal => self.make_normal(m.from, m.to),
            DoublePush => self.make_double_push(m.from, m.to, p),
            EnPassant => self.make_en_passant(m.from, m.to, p),
            Castle(castle_type) => self.make_castle(p, castle_type),
            Promotion(prom) => {
                if let Some(prom) = prom {
                    self.make_promotion(m.from, m.to, prom)
                } else {
                    log::error!("Promotion Move has no promotion piece assigned to it");
                    panic!();
                }
            }
        };

        // en passant state update
        if m.move_type != DoublePush {
            self.clear_ep_target();
        }
        // castling state update
        if m.from == Square::A1 || m.to == Square::A1 {
            self.castle_flags.unset_white_ooo();
        }
        if m.from == Square::A8 || m.to == Square::A8 {
            self.castle_flags.unset_black_ooo();
        }
        if m.from == Square::H1 || m.to == Square::H1 {
            self.castle_flags.unset_white_oo();
        }
        if m.from == Square::H8 || m.to == Square::H8 {
            self.castle_flags.unset_black_oo();
        }
        if m.from == Square::E1 || m.to == Square::E1 {
            self.castle_flags.unset_white_oo();
            self.castle_flags.unset_white_ooo();
        }
        if m.from == Square::E8 || m.to == Square::E8 {
            self.castle_flags.unset_black_oo();
            self.castle_flags.unset_black_ooo();
        }

        let castledelta = self.castle_flags.0 ^ prev_castle_flags.0;
        if self.active_color == Color::Black {
            self.fullmove_number += 1;
        }

        self.hash = update_castle(prev_castle_flags.raw(), self.hash);
        self.hash = update_castle(self.castle_flags.raw(), self.hash);

        self.halfmove_clock += 1;
        self.toggle_active_color();
        Some(MoveCommit::new(
            m,
            p,
            cap,
            prev_ep_target,
            CastleFlags(castledelta),
        ))
    }

    fn make_normal(&mut self, from: Square, to: Square) -> Option<BoardPiece> {
        let cap = self.remove_piece(to);
        self.move_piece(from, to);
        cap
    }

    fn make_double_push(&mut self, from: Square, to: Square, p: BoardPiece) -> Option<BoardPiece> {
        let pcolor = p.get_color();
        // self.remove_piece(to);
        self.move_piece(from, to);
        if pcolor == Color::White {
            self.set_ep_target(Square::try_from(to as usize - 8).unwrap());
        } else {
            self.set_ep_target(Square::try_from(to as usize + 8).unwrap());
        }
        None
    }

    fn make_en_passant(&mut self, from: Square, to: Square, p: BoardPiece) -> Option<BoardPiece> {
        let pcolor = p.get_color();
        // self.remove_piece(to);
        self.move_piece(from, to);
        if pcolor == Color::White {
            self.remove_piece(Square::try_from(to as usize - 8).unwrap())
        } else {
            self.remove_piece(Square::try_from(to as usize + 8).unwrap())
        }
    }

    fn make_castle(&mut self, p: BoardPiece, castle_type: CastleType) -> Option<BoardPiece> {
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
                self.castle_flags.unset_white_oo();
                self.castle_flags.unset_white_ooo();
            }
            Color::Black => {
                self.castle_flags.unset_black_oo();
                self.castle_flags.unset_black_ooo();
            }
        }

        None
    }

    fn make_promotion(&mut self, from: Square, to: Square, prom: BoardPiece) -> Option<BoardPiece> {
        let cap = self.remove_piece(to);
        self.remove_piece(from);
        self.add_piece(prom, to);
        cap
    }

    pub fn undo(&mut self) {
        if let Some(commit) = self.move_history.pop() {
            self.undo_commit(&commit);
        }
    }

    pub fn undo_commit(&mut self, commit: &MoveCommit) {
        let m = commit.m;
        let cap = commit.captured;
        let p = commit.p;
        let pcolor = p.get_color();

        use MoveType::*;
        match m.move_type {
            Normal => self.undo_normal(m.from, m.to, cap),
            DoublePush => self.undo_double_push(m.from, m.to),
            EnPassant => self.undo_en_passant(m.from, m.to, p, cap),
            Castle(castle_type) => self.undo_castle(p, castle_type),
            Promotion(prom) => {
                if let Some(_) = prom {
                    self.undo_promotion(m.from, m.to, p, cap);
                } else {
                    log::error!("Promotion Move has no promotion piece assigned to it");
                    panic!();
                }
            }
        }

        if pcolor == Color::Black {
            self.fullmove_number -= 1;
        }
        if let Some(t) = commit.ep_target {
            self.set_ep_target(t);
        } else {
            self.clear_ep_target();
        }
        let oldcastleflags = self.castle_flags.0 ^ commit.castledelta.0;
        self.hash = update_castle(self.castle_flags.raw(), self.hash);
        self.hash = update_castle(oldcastleflags, self.hash);
        self.castle_flags = CastleFlags(oldcastleflags);
        self.halfmove_clock -= 1;
        self.toggle_active_color();
    }

    fn undo_normal(&mut self, from: Square, to: Square, cap: Option<BoardPiece>) {
        self.move_piece(to, from);
        if let Some(cap) = cap {
            self.add_piece(cap, to);
        }
    }

    fn undo_double_push(&mut self, from: Square, to: Square) {
        self.move_piece(to, from);
    }

    fn undo_castle(&mut self, p: BoardPiece, castle_type: CastleType) {
        match p.get_color() {
            Color::White => match castle_type {
                CastleType::KingSide => {
                    self.move_piece(Square::G1, Square::E1);
                    self.move_piece(Square::F1, Square::H1);
                }
                CastleType::QueenSide => {
                    self.move_piece(Square::C1, Square::E1);
                    self.move_piece(Square::D1, Square::A1);
                }
            },
            Color::Black => match castle_type {
                CastleType::KingSide => {
                    self.move_piece(Square::G8, Square::E8);
                    self.move_piece(Square::F8, Square::H8);
                }
                CastleType::QueenSide => {
                    self.move_piece(Square::C8, Square::E8);
                    self.move_piece(Square::D8, Square::A8);
                }
            },
        }
    }

    fn undo_promotion(&mut self, from: Square, to: Square, p: BoardPiece, cap: Option<BoardPiece>) {
        self.remove_piece(to);
        match p.get_color() {
            Color::White => self.add_piece(BoardPiece::WhitePawn, from),
            Color::Black => self.add_piece(BoardPiece::BlackPawn, from),
        }
        if let Some(cap) = cap {
            self.add_piece(cap, to);
        }
    }

    fn undo_en_passant(
        &mut self,
        from: Square,
        to: Square,
        p: BoardPiece,
        cap: Option<BoardPiece>,
    ) {
        self.move_piece(to, from);
        if let Some(cap) = cap {
            let cap_sq = if p.get_color() == Color::White {
                Square::try_from(to as usize - 8).unwrap()
            } else {
                Square::try_from(to as usize + 8).unwrap()
            };
            self.add_piece(cap, cap_sq);
        }
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

    pub fn get_fen(&self) -> String {
        Fen::make_fen_from_config(self)
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
        self.castle_flags.can_white_ooo()
    }

    pub fn get_can_white_castle_kingside(&self) -> bool {
        self.castle_flags.can_white_oo()
    }

    pub fn get_can_black_castle_queenside(&self) -> bool {
        self.castle_flags.can_black_ooo()
    }

    pub fn get_can_black_castle_kingside(&self) -> bool {
        self.castle_flags.can_black_oo()
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

    pub fn get_castle_flags_raw(&self) -> u8 {
        self.castle_flags.raw()
    }

    fn move_piece(&mut self, from: Square, to: Square) {
        if let Some(p) = self.get_at_sq(from) {
            self.remove_piece(from);
            self.add_piece(p, to);
        } else {
            log::error!("No Piece at {}", from);
            log::error!("FEN {}", self.get_fen());
            self.print_board();
            panic!();
        }
    }

    fn remove_piece(&mut self, from: Square) -> Option<BoardPiece> {
        if let Some(p) = self.get_at_sq(from) {
            self.remove_from_bitboard(p, from);
            self.hash = update_piece(from, p, self.hash);
            Some(p)
        } else {
            None
        }
    }

    fn add_piece(&mut self, p: BoardPiece, to: Square) {
        self.hash = update_piece(to, p, self.hash);
        self.add_to_bitboard(p, to)
    }

    fn toggle_active_color(&mut self) {
        self.hash = update_side(self.active_color, self.hash);
        self.active_color = !self.active_color;
        self.hash = update_side(self.active_color, self.hash);
    }

    fn remove_from_bitboard(&mut self, p: BoardPiece, pos: Square) {
        self.bitboards[p as usize].unset(pos);
    }

    fn add_to_bitboard(&mut self, p: BoardPiece, pos: Square) {
        self.bitboards[p as usize].set(pos);
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct CastleFlags(u8);

impl CastleFlags {
    pub fn can_white_oo(&self) -> bool {
        self.0 & 1 > 0
    }

    pub fn set_white_oo(&mut self) {
        self.0 |= 1;
    }

    pub fn unset_white_oo(&mut self) {
        self.0 &= !(1);
    }

    pub fn can_white_ooo(&self) -> bool {
        self.0 & (1 << 1) > 0
    }

    pub fn set_white_ooo(&mut self) {
        self.0 |= 1 << 1;
    }

    pub fn unset_white_ooo(&mut self) {
        self.0 &= !(1 << 1);
    }

    pub fn can_black_oo(&self) -> bool {
        self.0 & (1 << 2) > 0
    }

    pub fn set_black_oo(&mut self) {
        self.0 |= 1 << 2;
    }

    pub fn unset_black_oo(&mut self) {
        self.0 &= !(1 << 2);
    }

    pub fn can_black_ooo(&self) -> bool {
        self.0 & (1 << 3) > 0
    }

    pub fn set_black_ooo(&mut self) {
        self.0 |= 1 << 3;
    }

    pub fn unset_black_ooo(&mut self) {
        self.0 &= !(1 << 3);
    }

    pub fn raw(&self) -> u8 {
        self.0
    }
}

#[derive(Debug)]
struct NoPieceErr;
