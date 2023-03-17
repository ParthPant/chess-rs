pub mod tables;

use crate::data::{bitboard::BitBoard, piece::BoardPiece, BoardConfig, Square};
use tables::*;

pub struct MoveGenerator {
    rook_magics: [MagicEntry; 64],
    bishop_magics: [MagicEntry; 64],

    rook_moves: Vec<Vec<BitBoard>>,
    bishop_moves: Vec<Vec<BitBoard>>,
}

impl Default for MoveGenerator {
    fn default() -> Self {
        let mut rook_magics = [MagicEntry::default(); 64];
        let mut bishop_magics = [MagicEntry::default(); 64];

        let mut rook_moves: Vec<Vec<BitBoard>> = vec![vec![]; 64];
        let mut bishop_moves: Vec<Vec<BitBoard>> = vec![vec![]; 64];

        log::info!("Generating Magic Entries");
        for i in 0..64 {
            let (bishop_magic, bishop_move_tbl) = find_magic(i, BoardPiece::WhiteBishop);
            bishop_magics[i] = bishop_magic;
            bishop_moves[i] = bishop_move_tbl.into();
            log::trace!(
                "Bishop Magic Entry for square {i}\nMagic: {:?}",
                bishop_magic
            );

            let (rook_magic, rook_move_tbl) = find_magic(i, BoardPiece::WhiteRook);
            rook_magics[i] = rook_magic;
            rook_moves[i] = rook_move_tbl;
            log::trace!("Rook Magic Entry for square {i}\nMagic: {:?}", rook_magic);
        }
        log::info!("Done Generating Magic Entires");

        MoveGenerator {
            rook_magics,
            bishop_magics,

            rook_moves,
            bishop_moves,
        }
    }
}

impl MoveGenerator {
    fn get_rook_atk(&self, sq: Square, blockers: BitBoard) -> BitBoard {
        let magic = self.rook_magics[sq as usize];
        let moves = &self.rook_moves[sq as usize];
        moves[magic_index(&magic, blockers)]
    }

    fn get_bishop_atk(&self, sq: Square, blockers: BitBoard) -> BitBoard {
        let magic = self.bishop_magics[sq as usize];
        let moves = &self.bishop_moves[sq as usize];
        moves[magic_index(&magic, blockers)]
    }

    fn get_queen_atk(&self, sq: Square, blockers: BitBoard) -> BitBoard {
        self.get_rook_atk(sq, blockers) | self.get_bishop_atk(sq, blockers)
    }

    fn get_white_pawn_atk(&self, sq: Square) -> BitBoard {
        WP_ATK_TBL[sq as usize].into()
    }

    fn get_black_pawn_atk(&self, sq: Square) -> BitBoard {
        BP_ATK_TBL[sq as usize].into()
    }

    fn get_knight_atk(&self, sq: Square) -> BitBoard {
        N_ATK_TBL[sq as usize].into()
    }

    fn get_king_atk(&self, sq: Square) -> BitBoard {
        K_ATK_TBL[sq as usize].into()
    }

    fn get_rook_moves(&self, sq: Square, blockers: BitBoard, friendly: BitBoard) -> BitBoard {
        self.get_rook_atk(sq, blockers) & !friendly
    }

    fn get_bishop_moves(&self, sq: Square, blockers: BitBoard, friendly: BitBoard) -> BitBoard {
        self.get_bishop_atk(sq, blockers) & !friendly
    }

    pub fn get_moves(&self, pos: Square, config: &BoardConfig) -> BitBoard {
        let piece = config.get_at_sq(pos).unwrap();
        use BoardPiece::*;
        match piece {
            WhiteRook => {
                let blockers = config.all_occupancy();
                let friendly = config.white_occupancy();
                self.get_rook_moves(pos, blockers, friendly)
            }
            BlackRook => {
                let blockers = config.all_occupancy();
                let friendly = config.black_occupancy();
                self.get_rook_moves(pos, blockers, friendly)
            }
            WhiteBishop => {
                let blockers = config.all_occupancy();
                let friendly = config.white_occupancy();
                self.get_bishop_moves(pos, blockers, friendly)
            }
            BlackBishop => {
                let blockers = config.all_occupancy();
                let friendly = config.black_occupancy();
                self.get_bishop_moves(pos, blockers, friendly)
            }
            WhiteKnight => {
                let friendly = config.white_occupancy();
                self.get_knight_atk(pos) & !friendly
            }
            BlackKnight => {
                let friendly = config.black_occupancy();
                self.get_knight_atk(pos) & !friendly
            }
            WhiteKing => {
                let friendly = config.white_occupancy();
                self.get_king_atk(pos) & !friendly
            }
            BlackKing => {
                let friendly = config.black_occupancy();
                self.get_king_atk(pos) & !friendly
            }
            WhiteQueen => {
                let blockers = config.all_occupancy();
                let friendly = config.white_occupancy();
                self.get_rook_moves(pos, blockers, friendly)
                    | self.get_bishop_moves(pos, blockers, friendly)
            }
            BlackQueen => {
                let blockers = config.all_occupancy();
                let friendly = config.black_occupancy();
                self.get_rook_moves(pos, blockers, friendly)
                    | self.get_bishop_moves(pos, blockers, friendly)
            }
            // TODO: Quite\Castling moves for pawns
            WhitePawn => {
                let enemy = config.black_occupancy();
                self.get_white_pawn_atk(pos) & enemy
            }
            BlackPawn => {
                let enemy = config.white_occupancy();
                self.get_black_pawn_atk(pos) & enemy
            }
        }
    }
}
