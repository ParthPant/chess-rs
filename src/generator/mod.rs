pub mod tables;

use tables::*;

use crate::data::{bitboard::BitBoard, piece::BoardPiece};

pub struct MoveGenerator {
    rook_magics: [MagicEntry; 64],
    bishop_magics: [MagicEntry; 64],

    rook_moves: Vec<Vec<u64>>,
    bishop_moves: Vec<Vec<u64>>,
}

impl Default for MoveGenerator {
    fn default() -> Self {
        let mut rook_magics = [MagicEntry::default(); 64];
        let mut bishop_magics = [MagicEntry::default(); 64];

        let mut rook_moves: Vec<Vec<u64>> = vec![vec![]; 64];
        let mut bishop_moves: Vec<Vec<u64>> = vec![vec![]; 64];

        log::info!("Generating Magic Entries");
        for i in 0..64 {
            let (bishop_magic, bishop_move_tbl) = find_magic(i, BoardPiece::WhiteBishop);
            bishop_magics[i] = bishop_magic;
            bishop_moves[i] = bishop_move_tbl;
            log::trace!(
                "Bishop Magic Entry for square {i}\nMagic: {:?}",
                bishop_magic
            );

            let (rook_magic, rook_move_tbl) = find_magic(i, BoardPiece::WhiteRook);
            rook_magics[i] = rook_magic;
            rook_moves[i] = rook_move_tbl;
            log::trace!("Rook Magic Entry for square {i}\nMagic: {:?}", rook_magic);
        }

        MoveGenerator {
            rook_magics,
            bishop_magics,

            rook_moves,
            bishop_moves,
        }
    }
}

impl MoveGenerator {
    pub fn get_rook_atk(&self, sq: usize, blockers: u64) -> u64 {
        let magic = self.rook_magics[sq];
        let moves = &self.rook_moves[sq];
        moves[magic_index(&magic, blockers)]
    }

    pub fn get_bishop_atk(&self, sq: usize, blockers: u64) -> u64 {
        let magic = self.bishop_magics[sq];
        let moves = &self.bishop_moves[sq];
        moves[magic_index(&magic, blockers)]
    }

    pub fn get_queen_atk(&self, sq: usize, blockers: u64) -> u64 {
        self.get_rook_atk(sq, blockers) | self.get_bishop_atk(sq, blockers)
    }

    pub fn get_white_pawn_atk(&self, sq: usize) -> u64 {
        WP_ATK_TBL[sq]
    }

    pub fn get_black_pawn_atk(&self, sq: usize) -> u64 {
        BP_ATK_TBL[sq]
    }

    pub fn get_knight_atk(&self, sq: usize) -> u64 {
        N_ATK_TBL[sq]
    }

    pub fn get_king_atk(&self, sq: usize) -> u64 {
        K_ATK_TBL[sq]
    }
}
