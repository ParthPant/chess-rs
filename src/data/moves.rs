use super::piece::BoardPiece;
use super::square::Square;
use super::BoardConfig;
use std::fmt::Debug;

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum CastleType {
    KingSide,
    QueenSide,
}

#[derive(Debug, PartialEq, Eq)]
pub enum MoveType {
    Normal,
    DoublePush,
    EnPassant,
    Castle(CastleType),
    Promotion(Option<BoardPiece>),
}

#[derive(Debug)]
pub struct Move {
    pub from: Square,
    pub to: Square,
    pub move_type: MoveType,
}

impl Move {
    pub fn new(from: Square, to: Square, m: MoveType) -> Self {
        Self {
            from,
            to,
            move_type: m,
        }
    }

    pub fn new_prom(from: Square, to: Square, p: BoardPiece) -> Self {
        Self {
            from,
            to,
            move_type: MoveType::Promotion(Some(p)),
        }
    }

    pub fn infer(from: Square, to: Square, c: &BoardConfig) -> Self {
        use MoveType::*;

        let p = c.get_at_sq(from).unwrap();
        let mut m: MoveType = Normal;

        // castling
        if p == BoardPiece::WhiteKing {
            if from == Square::E1 && to == Square::G1 {
                m = Castle(CastleType::KingSide);
            } else if from == Square::E1 && to == Square::C1 {
                m = Castle(CastleType::QueenSide);
            }
        } else if p == BoardPiece::BlackKing {
            if from == Square::E8 && to == Square::G8 {
                m = Castle(CastleType::KingSide);
            } else if from == Square::E8 && to == Square::C8 {
                m = Castle(CastleType::QueenSide);
            }
        }
        // double_push
        else if p == BoardPiece::WhitePawn {
            if to as usize - from as usize == 16 {
                m = DoublePush;
            } else if let Some(t) = c.en_passant_target {
                if to == t {
                    m = EnPassant;
                }
            } else if to > Square::H7 {
                // promotion
                m = Promotion(None);
            }
        } else if p == BoardPiece::BlackPawn {
            if from as usize - to as usize == 16 {
                m = DoublePush;
            } else if let Some(t) = c.en_passant_target {
                if to == t {
                    m = EnPassant;
                }
            } else if to < Square::A2 {
                // promotion
                m = Promotion(None)
            }
        }
        Self {
            from,
            to,
            move_type: m,
        }
    }

    pub fn is_prom(&self) -> bool {
        if let MoveType::Promotion(_) = self.move_type {
            return true;
        }
        false
    }

    pub fn set_prom(&mut self, p: BoardPiece) {
        if self.is_prom() {
            self.move_type = MoveType::Promotion(Some(p));
        }
    }
}

pub struct MoveCommit {
    pub m: Move,
    pub moved_piece: Option<BoardPiece>,
    pub captured: Option<BoardPiece>,
}

pub struct MoveList(Vec<Move>);

impl MoveList {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn add(&mut self, m: Move) {
        self.0.push(m);
    }

    pub fn has_target_sq(&self, sq: Square) -> bool {
        self.0.iter().any(|x| x.to == sq)
    }
}
