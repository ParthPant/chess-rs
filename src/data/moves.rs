use super::piece::BoardPiece;
use super::square::Square;
use super::{BoardConfig, Color, Fen};
use std::fmt::Debug;

pub struct MoveList(Vec<Move>);

#[derive(Debug)]
pub struct Move {
    pub from: Square,
    pub to: Square,
    pub m: Box<dyn IsMove>,
}

pub trait IsMove: Debug {
    fn apply(&self, to: Square, from: Square, p: BoardPiece, c: &mut BoardConfig);
}

impl MoveList {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn add(&mut self, m: Move) {
        self.0.push(m);
    }

    pub fn has_target_sq(&self, sq: Square) -> bool {
        self.0.iter().any(|x| x.to() == sq)
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct NormalMove {
    pub double_push: bool,
    pub ep: bool,
}

impl IsMove for NormalMove {
    fn apply(&self, to: Square, from: Square, p: BoardPiece, c: &mut BoardConfig) {
        let pcolor = p.get_color();
        c.remove_piece(to);
        c.move_piece(from, to);
        if self.double_push {
            if pcolor == Color::White {
                c.en_passant_target = Some(Square::try_from(to as usize - 8).unwrap());
            } else {
                c.en_passant_target = Some(Square::try_from(to as usize + 8).unwrap());
            }
        } else {
            c.en_passant_target = None;
        }

        if self.ep {
            c.en_passant_target = None;
            if pcolor == Color::White {
                c.remove_piece(Square::try_from(to as usize - 8).unwrap());
            } else {
                c.remove_piece(Square::try_from(to as usize + 8).unwrap());
            }
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum CastleType {
    KingSide,
    QueenSide,
}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct CastleMove {
    pub castle: CastleType,
}

impl IsMove for CastleMove {
    fn apply(&self, _to: Square, _from: Square, p: BoardPiece, c: &mut BoardConfig) {
        let pcolor = p.get_color();
        match self.castle {
            CastleType::KingSide => {
                if pcolor == Color::White {
                    c.move_piece(Square::E1, Square::G1);
                    c.move_piece(Square::H1, Square::F1);
                }
                if pcolor == Color::Black {
                    c.move_piece(Square::E8, Square::G8);
                    c.move_piece(Square::H8, Square::F8);
                }
            }
            CastleType::QueenSide => {
                if pcolor == Color::White {
                    c.move_piece(Square::E1, Square::C1);
                    c.move_piece(Square::A1, Square::D1);
                }
                if pcolor == Color::Black {
                    c.move_piece(Square::E8, Square::C8);
                    c.move_piece(Square::A8, Square::D8);
                }
            }
        }

        match pcolor {
            Color::White => {
                c.can_white_castle_kingside = false;
                c.can_white_castle_queenside = false;
            }
            Color::Black => {
                c.can_black_castle_kingside = false;
                c.can_black_castle_queenside = false;
            }
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct PromMove {
    pub prom: Option<BoardPiece>,
}

impl IsMove for PromMove {
    fn apply(&self, to: Square, from: Square, _p: BoardPiece, c: &mut BoardConfig) {
        if let Some(prom) = self.prom {
            c.remove_piece(from);
            c.add_piece(prom, to);
        }
    }
}

impl Move {
    pub fn new(from: Square, to: Square, m: Box<dyn IsMove>) -> Self {
        Self { from, to, m }
    }

    pub fn infer(from: Square, to: Square, c: &BoardConfig) -> Self {
        let p = c.get_at_sq(from).unwrap();
        let mut m: Box<dyn IsMove> = Box::from(NormalMove {
            double_push: false,
            ep: false,
        });

        // castling
        if p == BoardPiece::WhiteKing {
            if from == Square::E1 && to == Square::G1 {
                m = Box::from(CastleMove {
                    castle: CastleType::KingSide,
                });
            } else if from == Square::E1 && to == Square::C1 {
                m = Box::from(CastleMove {
                    castle: CastleType::QueenSide,
                });
            }
        } else if p == BoardPiece::BlackKing {
            if from == Square::E8 && to == Square::G8 {
                m = Box::from(CastleMove {
                    castle: CastleType::KingSide,
                });
            } else if from == Square::E8 && to == Square::C8 {
                m = Box::from(CastleMove {
                    castle: CastleType::QueenSide,
                });
            }
        }
        // double_push
        else if p == BoardPiece::WhitePawn {
            if to as usize - from as usize == 16 {
                m = Box::from(NormalMove {
                    double_push: true,
                    ep: false,
                })
            } else if let Some(t) = c.en_passant_target {
                if to == t {
                    m = Box::from(NormalMove {
                        double_push: false,
                        ep: true,
                    })
                }
            } else if to > Square::H7 {
                // promotion
                m = Box::from(PromMove {
                    prom: Some(BoardPiece::WhiteQueen),
                })
            }
        } else if p == BoardPiece::BlackPawn {
            if from as usize - to as usize == 16 {
                m = Box::from(NormalMove {
                    double_push: true,
                    ep: false,
                })
            } else if let Some(t) = c.en_passant_target {
                if to == t {
                    m = Box::from(NormalMove {
                        double_push: false,
                        ep: true,
                    })
                }
            } else if to < Square::A2 {
                // promotion
                m = Box::from(PromMove {
                    prom: Some(BoardPiece::BlackQueen),
                })
            }
        }
        Self { from, to, m }
    }

    pub fn from(&self) -> Square {
        self.from
    }

    pub fn to(&self) -> Square {
        self.to
    }

    pub fn apply(&self, c: &mut BoardConfig) {
        log::info!("{:?}", self);
        let p = c.get_at_sq(self.from).unwrap();
        let pcolor = p.get_color();

        // prevent from moving when its not their turn
        if pcolor != c.active_color {
            return;
        }
        if let Some(cap) = c.get_at_sq(self.to) {
            // prevent from moving to a square with piece of same color
            if cap.get_color() == pcolor {
                return;
            }
        }

        self.m.apply(self.to, self.from, p, c);

        if p == BoardPiece::WhiteRook {
            if self.from == Square::A1 {
                c.can_white_castle_queenside = false;
            } else if self.from == Square::H1 {
                c.can_white_castle_kingside = false;
            }
        } else if p == BoardPiece::BlackRook {
            if self.from == Square::A8 {
                c.can_black_castle_queenside = false;
            } else if self.from == Square::H8 {
                c.can_black_castle_kingside = false;
            }
        } else if p == BoardPiece::WhiteKing {
            c.can_white_castle_kingside = false;
            c.can_white_castle_queenside = false;
        } else if p == BoardPiece::BlackKing {
            c.can_black_castle_kingside = false;
            c.can_black_castle_queenside = false;
        }

        if pcolor == Color::Black {
            c.fullmove_number += 1;
        }
        c.halfmove_clock += 1;
        c.toggle_active_color();
        c.fen_str = Fen::make_fen_from_config(c);
    }
}
