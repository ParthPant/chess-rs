mod fen;

use crate::board::piece::{BoardPiece, Color};
use fen::Fen;

pub type BoardMatrix = [[Option<BoardPiece>; 8]; 8];

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
}

impl Default for BoardConfig {
    fn default() -> Self {
        Fen::make_config_from_str("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
    }
}

impl BoardConfig {
    pub fn from_fen_str(s: &str) -> Self {
        Fen::make_config_from_str(s)
    }

    pub fn get_at_xy(&self, x: usize, y: usize) -> Option<BoardPiece> {
        self.board_mat[y][x]
    }

    pub fn move_xy_to_xy(&mut self, prev: (usize, usize), new: (usize, usize)) {
        self.board_mat[new.1][new.0] = self.board_mat[prev.1][prev.0];
        self.board_mat[prev.1][prev.0] = None;
        self.fen_str = Fen::make_fen_from_config(&self);
        log::info!("Move {:?} to {:?}", prev, new);
        log::info!("Fen: {}", self.fen_str);
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
}
