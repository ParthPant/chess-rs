use crate::data::{BoardConfig, BoardPiece, Color, Square};
use crate::prng::*;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref PIECE_KEYS: [[u64; 12]; 64] = {
        let mut t = [[0; 12]; 64];
        let mut i = 0;
        while i < 64 {
            let mut p = 0;
            while p < 12 {
                t[i][p] = random_u64();
                p += 1;
            }
            i += 1;
        }
        t
    };
    pub static ref EP_KEYS: [u64; 64] = {
        let mut t = [0; 64];
        let mut i = 0;
        while i < 64 {
            t[i] = random_u64();
            i += 1;
        }
        t
    };
    pub static ref CASTLE_KEYS: [u64; 16] = {
        let mut t = [0; 16];
        let mut i = 0;
        while i < 16 {
            t[i] = random_u64();
            i += 1;
        }
        t
    };
    pub static ref BLACK_TO_MOVE: u64 = random_u64();
}

pub fn hash(config: &BoardConfig) -> u64 {
    let mut key: u64 = 0;
    for i in 0..64 {
        if let Some(p) = config.get_at_sq(Square::try_from(i).unwrap()) {
            key ^= PIECE_KEYS[i][p as usize];
        }
    }
    if let Some(t) = config.get_en_passant_target() {
        key ^= EP_KEYS[t as usize];
    }
    key ^= CASTLE_KEYS[config.get_castle_flags_raw() as usize];
    if config.get_active_color() == Color::Black {
        key ^= *BLACK_TO_MOVE;
    }
    key
}

pub fn update_piece(sq: Square, p: BoardPiece, key: &mut u64) {
    *key ^= PIECE_KEYS[sq as usize][p as usize];
}

pub fn update_ep(sq: Square, key: &mut u64) {
    *key ^= EP_KEYS[sq as usize];
}

pub fn update_castle(c: u8, key: &mut u64) {
    *key ^= CASTLE_KEYS[c as usize];
}

pub fn update_side(side: Color, key: &mut u64) {
    if side == Color::White {
        *key ^= *BLACK_TO_MOVE;
    }
}
