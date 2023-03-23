use super::piece::BoardPiece;
use super::square::Square;
use super::{BoardConfig, CastleFlags};
use std::fmt::{Debug, Display, Formatter, Result};

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum CastleType {
    KingSide,
    QueenSide,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum MoveType {
    Normal,
    DoublePush,
    EnPassant,
    Castle(CastleType),
    Promotion(Option<BoardPiece>),
}

#[derive(Debug, Copy, Clone)]
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

    pub fn is_empty_prom(&self) -> bool {
        if let MoveType::Promotion(p) = self.move_type {
            return p == None;
        }
        false
    }

    pub fn set_prom(&mut self, p: BoardPiece) {
        if self.is_prom() {
            self.move_type = MoveType::Promotion(Some(p));
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct MoveCommit {
    pub m: Move,
    pub p: BoardPiece,
    pub captured: Option<BoardPiece>,
    pub ep_target: Option<Square>,
    pub castledelta: CastleFlags,
}

impl Display for MoveCommit {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        use MoveType::*;
        if self.m.move_type == EnPassant {
            return write!(f, "{}{}e.p.", self.m.from, self.m.to);
        } else if let Some(_) = self.captured {
            return write!(f, "{}{}x{}", self.p, self.m.from, self.m.to);
        } else if self.m.move_type == Castle(CastleType::KingSide) {
            return write!(f, "0-0");
        } else if self.m.move_type == Castle(CastleType::QueenSide) {
            return write!(f, "0-0-0");
        } else if let Promotion(Some(p)) = self.m.move_type {
            return write!(f, "{}{}", self.m.to, p);
        } else {
            return write!(f, "{}{}{}", self.p, self.m.from, self.m.to);
        }
    }
}

impl MoveCommit {
    pub fn new(
        m: Move,
        p: BoardPiece,
        captured: Option<BoardPiece>,
        ep_target: Option<Square>,
        castledelta: CastleFlags,
    ) -> Self {
        Self {
            m,
            p,
            captured,
            ep_target,
            castledelta,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MoveHistory {
    list: Box<[Option<MoveCommit>; 255]>,
    counter: usize,
    capacity: usize,
}

impl Default for MoveHistory {
    fn default() -> Self {
        MoveHistory {
            list: Box::new([None; 255]),
            counter: 0,
            capacity: 255,
        }
    }
}

impl MoveHistory {
    pub fn push(&mut self, m: MoveCommit) {
        if self.counter == self.capacity {
            log::error!("MoveHistory is out of capacity");
            panic!();
        }
        self.counter += 1;
        self.list[self.counter] = Some(m);
    }

    pub fn pop(&mut self) -> Option<MoveCommit> {
        let r = self.list[self.counter];
        self.list[self.counter] = None;
        if self.counter > 0 {
            self.counter -= 1;
        }
        r
    }
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

    pub fn len(&self) -> usize {
        self.0.len()
    }
}
