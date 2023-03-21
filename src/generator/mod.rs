pub mod tables;

use crate::data::{BitBoard, BoardConfig, BoardPiece, Color, Move, MoveList, Square};
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
    pub fn gen_piece_moves(
        &self,
        piece: BoardPiece,
        pos: Square,
        config: &BoardConfig,
    ) -> MoveList {
        use BoardPiece::*;
        match piece {
            WhiteRook => {
                let blockers = config.all_occupancy();
                let friendly = config.white_occupancy();
                let moves = self.get_rook_moves(pos, blockers, friendly);
                Self::make_movelist(moves, pos, config)
            }
            BlackRook => {
                let blockers = config.all_occupancy();
                let friendly = config.black_occupancy();
                let moves = self.get_rook_moves(pos, blockers, friendly);
                Self::make_movelist(moves, pos, config)
            }
            WhiteBishop => {
                let blockers = config.all_occupancy();
                let friendly = config.white_occupancy();
                let moves = self.get_bishop_moves(pos, blockers, friendly);
                Self::make_movelist(moves, pos, config)
            }
            BlackBishop => {
                let blockers = config.all_occupancy();
                let friendly = config.black_occupancy();
                let moves = self.get_bishop_moves(pos, blockers, friendly);
                Self::make_movelist(moves, pos, config)
            }
            WhiteKnight => {
                let friendly = config.white_occupancy();
                let moves = self.get_knight_atk(pos) & !friendly;
                Self::make_movelist(moves, pos, config)
            }
            BlackKnight => {
                let friendly = config.black_occupancy();
                let moves = self.get_knight_atk(pos) & !friendly;
                Self::make_movelist(moves, pos, config)
            }
            WhiteQueen => {
                let blockers = config.all_occupancy();
                let friendly = config.white_occupancy();
                let moves = self.get_rook_moves(pos, blockers, friendly)
                    | self.get_bishop_moves(pos, blockers, friendly);
                Self::make_movelist(moves, pos, config)
            }
            BlackQueen => {
                let blockers = config.all_occupancy();
                let friendly = config.black_occupancy();
                let moves = self.get_rook_moves(pos, blockers, friendly)
                    | self.get_bishop_moves(pos, blockers, friendly);
                Self::make_movelist(moves, pos, config)
            }
            WhiteKing => {
                let friendly = config.white_occupancy();
                let all = config.all_occupancy();
                let mut moves = self.get_king_atk(pos) & !friendly;
                if config.get_can_white_castle_kingside() {
                    if !(all.is_set(Square::F1) || all.is_set(Square::G1))
                        && !self.is_sq_attacked(Square::F1, Color::Black, config)
                    {
                        moves.set(Square::G1);
                    }
                }
                if config.get_can_white_castle_queenside() {
                    if !(all.is_set(Square::B1) || all.is_set(Square::C1) || all.is_set(Square::D1))
                        && !self.is_sq_attacked(Square::D1, Color::Black, config)
                    {
                        moves.set(Square::C1);
                    }
                }
                Self::make_movelist(moves, pos, config)
            }
            BlackKing => {
                let friendly = config.black_occupancy();
                let all = config.all_occupancy();
                let mut moves = self.get_king_atk(pos) & !friendly;
                if config.get_can_black_castle_kingside() {
                    if !(all.is_set(Square::F8) || all.is_set(Square::G8))
                        && !self.is_sq_attacked(Square::F8, Color::White, config)
                    {
                        moves.set(Square::G8);
                    }
                }
                if config.get_can_black_castle_queenside() {
                    if !(all.is_set(Square::B8) || all.is_set(Square::C8) || all.is_set(Square::D8))
                        && !self.is_sq_attacked(Square::D8, Color::White, config)
                    {
                        moves.set(Square::C8);
                    }
                }
                Self::make_movelist(moves, pos, config)
            }
            // TODO: Pawn Promotion
            WhitePawn => {
                let friendly = config.white_occupancy();
                let enemy = config.black_occupancy();
                let quiet = {
                    if pos < Square::H7 {
                        // not in rank 8
                        let single = BitBoard::from(1 << (pos as usize + 8)) & !friendly & !enemy;
                        if pos >= Square::A2 && pos <= Square::H2 && single > BitBoard::from(0) {
                            single | BitBoard::from(1 << (pos as usize + 16))
                        } else {
                            single
                        }
                    } else {
                        BitBoard::from(0)
                    }
                };
                let atks = self.get_white_pawn_atk(pos);
                let mut moves = (quiet & !friendly & !enemy) | (atks & enemy);
                if let Some(t) = config.get_en_passant_target() {
                    if atks & BitBoard::from(1 << t as usize) > BitBoard::from(0) {
                        moves |= BitBoard::from(1 << t as usize);
                    }
                }
                Self::make_movelist(moves, pos, config)
            }
            BlackPawn => {
                let friendly = config.black_occupancy();
                let enemy = config.white_occupancy();
                let quiet = {
                    if pos > Square::A2 {
                        // not in rank 1
                        let single = BitBoard::from(1 << (pos as usize - 8)) & !friendly & !enemy;
                        if pos >= Square::A7 && pos <= Square::H7 && single > BitBoard::from(0) {
                            single | BitBoard::from(1 << (pos as usize - 16))
                        } else {
                            single
                        }
                    } else {
                        BitBoard::from(0)
                    }
                };
                let atks = self.get_black_pawn_atk(pos);
                let mut moves = (quiet & !friendly & !enemy) | (atks & enemy);
                if let Some(t) = config.get_en_passant_target() {
                    if atks & BitBoard::from(1 << t as usize) > BitBoard::from(0) {
                        moves |= BitBoard::from(1 << t as usize);
                    }
                }
                Self::make_movelist(moves, pos, config)
            }
        }
    }

    fn is_sq_attacked(&self, sq: Square, color: Color, config: &BoardConfig) -> bool {
        match color {
            Color::White => {
                if self.get_black_pawn_atk(sq) & config.get_piece_occupancy(BoardPiece::WhitePawn)
                    > BitBoard::from(0)
                {
                    return true;
                } else if self.get_knight_atk(sq)
                    & config.get_piece_occupancy(BoardPiece::WhiteKnight)
                    > BitBoard::from(0)
                {
                    return true;
                } else if self.get_king_atk(sq) & config.get_piece_occupancy(BoardPiece::WhiteKing)
                    > BitBoard::from(0)
                {
                    return true;
                } else if self.get_bishop_atk(sq, config.all_occupancy())
                    & (config.get_piece_occupancy(BoardPiece::WhiteBishop)
                        | config.get_piece_occupancy(BoardPiece::WhiteQueen))
                    > BitBoard::from(0)
                {
                    return true;
                } else if self.get_rook_atk(sq, config.all_occupancy())
                    & (config.get_piece_occupancy(BoardPiece::WhiteRook)
                        | config.get_piece_occupancy(BoardPiece::WhiteQueen))
                    > BitBoard::from(0)
                {
                    return true;
                } else {
                    return false;
                }
            }
            Color::Black => {
                if self.get_white_pawn_atk(sq) & config.get_piece_occupancy(BoardPiece::BlackPawn)
                    > BitBoard::from(0)
                {
                    return true;
                } else if self.get_knight_atk(sq)
                    & config.get_piece_occupancy(BoardPiece::BlackKnight)
                    > BitBoard::from(0)
                {
                    return true;
                } else if self.get_king_atk(sq) & config.get_piece_occupancy(BoardPiece::BlackKing)
                    > BitBoard::from(0)
                {
                    return true;
                } else if self.get_bishop_atk(sq, config.all_occupancy())
                    & (config.get_piece_occupancy(BoardPiece::BlackBishop)
                        | config.get_piece_occupancy(BoardPiece::BlackQueen))
                    > BitBoard::from(0)
                {
                    return true;
                } else if self.get_rook_atk(sq, config.all_occupancy())
                    & (config.get_piece_occupancy(BoardPiece::BlackRook)
                        | config.get_piece_occupancy(BoardPiece::BlackQueen))
                    > BitBoard::from(0)
                {
                    return true;
                } else {
                    return false;
                }
            }
        }
    }

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

    fn make_movelist(mut moves: BitBoard, from: Square, c: &BoardConfig) -> MoveList {
        let mut list = MoveList::new();
        while moves.data() > 0 {
            let to = moves.pop_sq().unwrap();
            let m = Move::infer(from, to, c);
            if m.is_prom() {
                let p = c.get_at_sq(from).unwrap();
                use BoardPiece::*;
                match p.get_color() {
                    Color::White => {
                        list.add(Move::new_prom(from, to, WhiteRook));
                        list.add(Move::new_prom(from, to, WhiteBishop));
                        list.add(Move::new_prom(from, to, WhiteKnight));
                        list.add(Move::new_prom(from, to, WhiteQueen));
                    }
                    Color::Black => {
                        list.add(Move::new_prom(from, to, BlackRook));
                        list.add(Move::new_prom(from, to, BlackBishop));
                        list.add(Move::new_prom(from, to, BlackKnight));
                        list.add(Move::new_prom(from, to, BlackQueen));
                    }
                }
            } else {
                list.add(m);
            }
        }
        list
    }
}
