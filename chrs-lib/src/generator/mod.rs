pub mod tables;

use crate::data::{
    BitBoard, BoardConfig, BoardPiece, Color, Move, MoveList, MoveType, Square, B_PIECES, W_PIECES,
};
use tables::*;

pub struct MoveGenerator {
    rook_magics: [MagicEntry; 64],
    bishop_magics: [MagicEntry; 64],

    rook_moves: Vec<BitBoard>,
    bishop_moves: Vec<BitBoard>,
}

impl Default for MoveGenerator {
    fn default() -> Self {
        let mut rook_magics = [MagicEntry::default(); 64];
        let mut bishop_magics = [MagicEntry::default(); 64];

        let mut rook_moves: Vec<BitBoard> = vec![];
        let mut bishop_moves: Vec<BitBoard> = vec![];

        log::info!("Generating Magic Entries");
        for i in 0..64 {
            let (mut bishop_magic, mut bishop_move_tbl) = find_magic(i, BoardPiece::WhiteBishop);
            bishop_magic.offset = bishop_moves.len();
            bishop_magics[i] = bishop_magic;
            bishop_moves.append(&mut bishop_move_tbl);
            log::trace!(
                "Bishop Magic Entry for square {i}\nMagic: {:?}",
                bishop_magic
            );

            let (mut rook_magic, mut rook_move_tbl) = find_magic(i, BoardPiece::WhiteRook);
            rook_magic.offset = rook_moves.len();
            rook_magics[i] = rook_magic;
            rook_moves.append(&mut rook_move_tbl);
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
    pub fn update_state(&self, config: &mut BoardConfig) {
        let sq = match config.get_active_color() {
            Color::White => config.bitboards[BoardPiece::WhiteKing as usize].peek(),
            Color::Black => config.bitboards[BoardPiece::BlackKing as usize].peek(),
        };
        let is_attacked = self.is_sq_attacked(sq.unwrap(), !config.get_active_color(), config);
        let can_move = self
            .gen_all_moves(config.get_active_color(), config, false)
            .len()
            > 0;

        if is_attacked && !can_move {
            config.set_mate(config.get_active_color());
        } else if !can_move {
            config.set_stalemate();
        }
    }

    pub fn gen_all_moves(
        &self,
        side: Color,
        config: &mut BoardConfig,
        only_captures: bool,
    ) -> MoveList {
        let pieces = match side {
            Color::White => &W_PIECES,
            Color::Black => &B_PIECES,
        };

        let mut moves = MoveList::new();
        for p in pieces {
            let mut bb = config.bitboards[*p as usize];
            while *bb > 0 {
                let pos = bb.pop_sq().unwrap();
                self.gen_piece_moves_impl(*p, pos, config, only_captures, &mut moves);
            }
        }

        moves
    }

    pub fn gen_piece_moves(
        &self,
        piece: BoardPiece,
        pos: Square,
        config: &mut BoardConfig,
        only_captures: bool,
    ) -> MoveList {
        let mut list = MoveList::with_capacity(32);
        self.gen_piece_moves_impl(piece, pos, config, only_captures, &mut list);
        list
    }

    fn gen_piece_moves_impl(
        &self,
        piece: BoardPiece,
        pos: Square,
        config: &mut BoardConfig,
        only_captures: bool,
        list: &mut MoveList,
    ) {
        use BoardPiece::*;
        // let mut config = config.clone();
        let (friendly, enemy) = match piece.get_color() {
            Color::White => (config.white_occupancy(), config.black_occupancy()),
            Color::Black => (config.black_occupancy(), config.white_occupancy()),
        };
        let blockers = config.all_occupancy();
        let mut ep_moves = BitBoard::default();

        let moves = match piece {
            WhiteRook => self.get_rook_moves(pos, blockers, friendly),
            BlackRook => self.get_rook_moves(pos, blockers, friendly),
            WhiteBishop => self.get_bishop_moves(pos, blockers, friendly),
            BlackBishop => self.get_bishop_moves(pos, blockers, friendly),
            WhiteKnight => self.get_knight_atk(pos) & !friendly,
            BlackKnight => self.get_knight_atk(pos) & !friendly,
            WhiteQueen => {
                self.get_rook_moves(pos, blockers, friendly)
                    | self.get_bishop_moves(pos, blockers, friendly)
            }
            BlackQueen => {
                self.get_rook_moves(pos, blockers, friendly)
                    | self.get_bishop_moves(pos, blockers, friendly)
            }
            WhiteKing => {
                let all = config.all_occupancy();
                let mut moves = self.get_king_atk(pos) & !friendly;
                if pos == Square::E1 && config.get_can_white_castle_kingside() {
                    if !(all.is_set(Square::F1) || all.is_set(Square::G1))
                        && !self.is_sq_attacked(Square::F1, Color::Black, config)
                    {
                        moves.set(Square::G1);
                    }
                }
                if pos == Square::E1 && config.get_can_white_castle_queenside() {
                    if !(all.is_set(Square::B1) || all.is_set(Square::C1) || all.is_set(Square::D1))
                        && !self.is_sq_attacked(Square::D1, Color::Black, config)
                    {
                        moves.set(Square::C1);
                    }
                }
                moves
            }
            BlackKing => {
                let all = config.all_occupancy();
                let mut moves = self.get_king_atk(pos) & !friendly;
                if pos == Square::E8 && config.get_can_black_castle_kingside() {
                    if !(all.is_set(Square::F8) || all.is_set(Square::G8))
                        && !self.is_sq_attacked(Square::F8, Color::White, config)
                    {
                        moves.set(Square::G8);
                    }
                }
                if pos == Square::E8 && config.get_can_black_castle_queenside() {
                    if !(all.is_set(Square::B8) || all.is_set(Square::C8) || all.is_set(Square::D8))
                        && !self.is_sq_attacked(Square::D8, Color::White, config)
                    {
                        moves.set(Square::C8);
                    }
                }
                moves
            }
            WhitePawn => {
                let quiet = {
                    if pos < Square::A8 {
                        // not in rank 8
                        let single = BitBoard::from(1 << (pos as usize + 8)) & !friendly & !enemy;
                        if pos >= Square::A2 && pos <= Square::H2 && single.non_zero() {
                            (single | BitBoard::from(1 << (pos as usize + 16))) & !friendly & !enemy
                        } else {
                            single
                        }
                    } else {
                        BitBoard::from(0)
                    }
                };
                let atks = self.get_white_pawn_atk(pos);
                let moves = quiet | (atks & enemy);
                if let Some(t) = config.get_en_passant_target() {
                    if *atks & (1 << t as usize) > 0 {
                        *ep_moves |= 1 << t as usize;
                    }
                }
                moves
            }
            BlackPawn => {
                let quiet = {
                    if pos > Square::H1 {
                        // not in rank 1
                        let single = BitBoard::from(1 << (pos as usize - 8)) & !friendly & !enemy;
                        if pos >= Square::A7 && pos <= Square::H7 && single.non_zero() {
                            (single | BitBoard::from(1 << (pos as usize - 16))) & !friendly & !enemy
                        } else {
                            single
                        }
                    } else {
                        BitBoard::from(0)
                    }
                };
                let atks = self.get_black_pawn_atk(pos);
                let moves = quiet | (atks & enemy);
                if let Some(t) = config.get_en_passant_target() {
                    if *atks & (1 << t as usize) > 0 {
                        *ep_moves |= 1 << t as usize;
                    }
                }
                moves
            }
        };

        if only_captures {
            self.make_movelist((moves & enemy) | ep_moves, pos, config, list)
        } else {
            self.make_movelist(moves | ep_moves, pos, config, list)
        }
    }

    pub fn is_sq_attacked(&self, sq: Square, color: Color, config: &BoardConfig) -> bool {
        match color {
            Color::White => {
                if (self.get_black_pawn_atk(sq) & config.get_piece_occupancy(BoardPiece::WhitePawn))
                    .non_zero()
                {
                    return true;
                } else if (self.get_knight_atk(sq)
                    & config.get_piece_occupancy(BoardPiece::WhiteKnight))
                .non_zero()
                {
                    return true;
                } else if (self.get_king_atk(sq)
                    & config.get_piece_occupancy(BoardPiece::WhiteKing))
                .non_zero()
                {
                    return true;
                } else if (self.get_bishop_atk(sq, config.all_occupancy())
                    & (config.get_piece_occupancy(BoardPiece::WhiteBishop)
                        | config.get_piece_occupancy(BoardPiece::WhiteQueen)))
                .non_zero()
                {
                    return true;
                } else if (self.get_rook_atk(sq, config.all_occupancy())
                    & (config.get_piece_occupancy(BoardPiece::WhiteRook)
                        | config.get_piece_occupancy(BoardPiece::WhiteQueen)))
                .non_zero()
                {
                    return true;
                } else {
                    return false;
                }
            }
            Color::Black => {
                if (self.get_white_pawn_atk(sq) & config.get_piece_occupancy(BoardPiece::BlackPawn))
                    .non_zero()
                {
                    return true;
                } else if (self.get_knight_atk(sq)
                    & config.get_piece_occupancy(BoardPiece::BlackKnight))
                .non_zero()
                {
                    return true;
                } else if (self.get_king_atk(sq)
                    & config.get_piece_occupancy(BoardPiece::BlackKing))
                .non_zero()
                {
                    return true;
                } else if (self.get_bishop_atk(sq, config.all_occupancy())
                    & (config.get_piece_occupancy(BoardPiece::BlackBishop)
                        | config.get_piece_occupancy(BoardPiece::BlackQueen)))
                .non_zero()
                {
                    return true;
                } else if (self.get_rook_atk(sq, config.all_occupancy())
                    & (config.get_piece_occupancy(BoardPiece::BlackRook)
                        | config.get_piece_occupancy(BoardPiece::BlackQueen)))
                .non_zero()
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
        // let moves = &self.rook_moves[sq as usize];
        self.rook_moves[magic_index(&magic, blockers)]
    }

    fn get_bishop_atk(&self, sq: Square, blockers: BitBoard) -> BitBoard {
        let magic = self.bishop_magics[sq as usize];
        // let moves = &self.bishop_moves[sq as usize];
        self.bishop_moves[magic_index(&magic, blockers)]
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

    fn is_legal(&self, m: Move, c: &mut BoardConfig, side: Color) -> bool {
        if let Some(commit) = c.make_move(m) {
            let king_sq = match side {
                Color::White => c.bitboards[BoardPiece::WhiteKing as usize].peek().unwrap(),
                Color::Black => c.bitboards[BoardPiece::BlackKing as usize].peek().unwrap(),
            };
            let res = !self.is_sq_attacked(king_sq, !side, c);
            c.undo_commit(&commit);
            return res;
        }
        false
    }

    fn make_movelist(
        &self,
        mut moves: BitBoard,
        from: Square,
        config: &mut BoardConfig,
        list: &mut MoveList,
    ) {
        while *moves > 0 {
            let to = moves.pop_sq().unwrap();
            let m = Move::infer(from, to, config);
            let p = m.p;
            if m.is_prom() {
                use BoardPiece::*;
                match p.get_color() {
                    Color::White => {
                        let m = Move::new_prom(from, to, p, m.capture, WhiteRook);
                        if self.is_legal(m, config, p.get_color()) {
                            list.push(m);
                            list.push(Move::new_prom(from, to, p, m.capture, WhiteBishop));
                            list.push(Move::new_prom(from, to, p, m.capture, WhiteKnight));
                            list.push(Move::new_prom(from, to, p, m.capture, WhiteQueen));
                        }
                    }
                    Color::Black => {
                        let m = Move::new_prom(from, to, p, m.capture, BlackRook);
                        if self.is_legal(m, config, p.get_color()) {
                            list.push(Move::new_prom(from, to, p, m.capture, BlackBishop));
                            list.push(Move::new_prom(from, to, p, m.capture, BlackKnight));
                            list.push(Move::new_prom(from, to, p, m.capture, BlackQueen));
                        }
                    }
                }
            } else {
                if let MoveType::Castle(_) = m.move_type {
                    if self.is_sq_attacked(from, !p.get_color(), config) {
                        continue;
                    }
                }
                if self.is_legal(m, config, p.get_color()) {
                    list.push(m);
                }
            }
        }
    }
}
