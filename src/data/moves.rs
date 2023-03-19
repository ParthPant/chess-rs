use super::piece::BoardPiece;
use super::square::Square;
use super::BoardConfig;

pub struct MoveList(Vec<Move>);

impl MoveList {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn add(&mut self, m: Move) {
        self.0.push(m);
    }

    pub fn has_move(&self, m: Move) -> bool {
        self.0.iter().any(|&x| x == m)
    }

    pub fn has_target_sq(&self, sq: Square) -> bool {
        self.0.iter().any(|&x| x.to == sq)
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct Move {
    pub from: Square,
    pub to: Square,
    pub ep: bool,
    pub castle_oo: bool,
    pub castle_ooo: bool,
    pub double_push: bool,
    pub promotion: Option<BoardPiece>,
}

impl Move {
    pub fn new(
        from: Square,
        to: Square,
        ep: bool,
        castle_oo: bool,
        castle_ooo: bool,
        double_push: bool,
        promotion: Option<BoardPiece>,
    ) -> Self {
        Self {
            from,
            to,
            ep,
            castle_oo,
            castle_ooo,
            double_push,
            promotion,
        }
    }

    pub fn infer(from: Square, to: Square, c: &BoardConfig) -> Self {
        let p = c.get_at_sq(from).unwrap();
        let mut castle_oo = false;
        let mut castle_ooo = false;
        let mut double_push = false;
        let mut ep = false;
        let mut promotion = None;

        // castling
        if p == BoardPiece::WhiteKing {
            if from == Square::E1 && to == Square::G1 {
                castle_oo = true;
            } else if from == Square::E1 && to == Square::C1 {
                castle_ooo = true;
            }
        } else if p == BoardPiece::BlackKing {
            if from == Square::E8 && to == Square::G8 {
                castle_oo = true;
            } else if from == Square::E8 && to == Square::C8 {
                castle_ooo = true;
            }
        }

        // double_push
        if p == BoardPiece::WhitePawn {
            if to as usize - from as usize == 16 {
                double_push = true;
            }
        } else if p == BoardPiece::BlackPawn {
            if from as usize - to as usize == 16 {
                double_push = true;
            }
        }

        // promotion
        if p == BoardPiece::WhitePawn {
            if to > Square::H7 {
                promotion = Some(BoardPiece::WhiteQueen);
            }
        } else if p == BoardPiece::BlackPawn {
            if to < Square::A2 {
                promotion = Some(BoardPiece::BlackQueen)
            }
        }

        // enpassant
        if p == BoardPiece::WhitePawn || p == BoardPiece::BlackPawn {
            if let Some(t) = c.en_passant_target {
                ep = t == to;
            }
        }

        Self {
            from,
            to,
            ep,
            castle_oo,
            castle_ooo,
            double_push,
            promotion,
        }
    }

    pub fn from(&self) -> Square {
        self.from
    }

    pub fn to(&self) -> Square {
        self.to
    }
}
