use super::piece::BoardPiece;
use super::square::Square;
use super::{BoardConfig, CastleFlags};
use std::fmt::{Debug, Display, Formatter, Result};
use std::ops::{Deref, DerefMut};

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

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Move {
    pub from: Square,
    pub to: Square,
    pub p: BoardPiece,
    pub capture: Option<BoardPiece>,
    pub move_type: MoveType,
}

impl Default for Move {
    fn default() -> Self {
        Self {
            from: Square::A1,
            to: Square::A1,
            p: BoardPiece::WhitePawn,
            capture: None,
            move_type: MoveType::EnPassant,
        }
    }
}

impl Display for Move {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self.move_type {
            MoveType::Promotion(Some(prom)) => write!(f, "{}{}{}", self.from, self.to, prom),
            _ => write!(f, "{}{}", self.from, self.to),
        }
    }
}

impl Move {
    pub fn new(
        from: Square,
        to: Square,
        p: BoardPiece,
        capture: Option<BoardPiece>,
        m: MoveType,
    ) -> Self {
        Self {
            from,
            to,
            p,
            capture,
            move_type: m,
        }
    }

    pub fn new_prom(
        from: Square,
        to: Square,
        p: BoardPiece,
        capture: Option<BoardPiece>,
        prom: BoardPiece,
    ) -> Self {
        Self {
            from,
            to,
            p,
            capture,
            move_type: MoveType::Promotion(Some(prom)),
        }
    }

    pub fn infer(from: Square, to: Square, c: &BoardConfig) -> Self {
        use MoveType::*;

        let p = c.get_at_sq(from).unwrap();
        let mut move_type: MoveType = Normal;
        let mut capture = c.get_at_sq(to);

        // Castling
        if p == BoardPiece::WhiteKing {
            if from == Square::E1 && to == Square::G1 {
                move_type = Castle(CastleType::KingSide);
            } else if from == Square::E1 && to == Square::C1 {
                move_type = Castle(CastleType::QueenSide);
            }
        } else if p == BoardPiece::BlackKing {
            if from == Square::E8 && to == Square::G8 {
                move_type = Castle(CastleType::KingSide);
            } else if from == Square::E8 && to == Square::C8 {
                move_type = Castle(CastleType::QueenSide);
            }
        }
        // Pawn: Double Push, En Passant and Promotion
        else if p == BoardPiece::WhitePawn {
            if to as usize - from as usize == 16 {
                move_type = DoublePush;
            } else if let Some(t) = c.en_passant_target {
                if to == t {
                    capture = c.get_at_sq(Square::try_from(t as usize - 8).unwrap());
                    move_type = EnPassant;
                }
            }
            if to >= Square::A8 {
                move_type = Promotion(None);
            }
        } else if p == BoardPiece::BlackPawn {
            if from as usize - to as usize == 16 {
                move_type = DoublePush;
            } else if let Some(t) = c.en_passant_target {
                if to == t {
                    capture = c.get_at_sq(Square::try_from(t as usize + 8).unwrap());
                    move_type = EnPassant;
                }
            }
            if to <= Square::H1 {
                move_type = Promotion(None)
            }
        }
        Self {
            from,
            to,
            p,
            capture,
            move_type,
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
    pub ep_target: Option<Square>,
    pub castledelta: CastleFlags,
}

impl Display for MoveCommit {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        use MoveType::*;
        if self.m.move_type == EnPassant {
            return write!(f, "{}{}{}e.p.", self.m.p.utf_str(), self.m.from, self.m.to);
        } else if let Some(_) = self.m.capture {
            return write!(f, "{}{}x{}", self.m.p.utf_str(), self.m.from, self.m.to);
        } else if self.m.move_type == Castle(CastleType::KingSide) {
            return write!(f, "0-0");
        } else if self.m.move_type == Castle(CastleType::QueenSide) {
            return write!(f, "0-0-0");
        } else if let Promotion(Some(prom)) = self.m.move_type {
            return write!(f, "{}{}", self.m.to, prom.utf_str());
        } else {
            return write!(f, "{}{}{}", self.m.p.utf_str(), self.m.from, self.m.to);
        }
    }
}

impl MoveCommit {
    pub fn new(m: Move, ep_target: Option<Square>, castledelta: CastleFlags) -> Self {
        Self {
            m,
            ep_target,
            castledelta,
        }
    }
}

#[derive(Debug, Clone)]
pub struct List<T> {
    pub list: [Option<T>; 255],
    pub counter: u8,
    capacity: u8,
}

impl<T: Copy> Default for List<T> {
    fn default() -> Self {
        List {
            list: [None; 255],
            counter: 0,
            capacity: 255,
        }
    }
}

impl<T: Copy> List<T> {
    pub fn clear(&mut self) {
        self.counter = 0;
    }

    pub fn push(&mut self, m: T) {
        if self.counter == self.capacity {
            log::error!("MoveHistory is out of capacity");
            panic!();
        }
        self.list[self.counter as usize] = Some(m);
        self.counter += 1;
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.counter > 0 {
            self.counter -= 1;
        }
        let r = self.list[self.counter as usize];
        self.list[self.counter as usize] = None;
        r
    }

    pub fn get_last(&self) -> Option<T> {
        self.list[self.counter as usize]
    }

    pub fn data(&mut self) -> &mut [Option<T>] {
        &mut self.list[..(self.counter as usize)]
    }

    pub fn len(&self) -> usize {
        self.counter as usize
    }

    pub fn iter(&self) -> impl Iterator<Item = &Option<T>> {
        self.list.iter().take(self.counter as usize)
    }
}

impl List<Move> {
    pub fn has_target_sq(&self, sq: Square) -> bool {
        self.list
            .iter()
            .take(self.counter as usize)
            .any(|x| x.unwrap().to == sq)
    }
}

pub struct MoveList(pub Vec<Move>);

impl Deref for MoveList {
    type Target = Vec<Move>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for MoveList {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl MoveList {
    pub fn has_target_sq(&self, sq: Square) -> bool {
        self.0.iter().any(|x| x.to == sq)
    }

    pub fn new() -> Self {
        Self(Vec::with_capacity(10))
    }
}
