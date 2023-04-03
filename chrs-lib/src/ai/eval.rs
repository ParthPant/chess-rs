use crate::data::{BoardConfig, BoardPiece, Color, Move, Square};
use strum::IntoEnumIterator;

const MATERIAL_SCORE: [i32; 12] = [
    100, 300, 350, 500, 1000, 10000, -100, -300, -350, -500, -1000, -10000,
];

#[rustfmt::skip]
const PAWN_SCORE: [i32; 64] =
[
     0,   0,   0,   0,   0,   0,   0,   0,
     0,   0,   0, -10, -10,   0,   0,   0,
     0,   0,   0,   5,   5,   0,   0,   0,
     5,   5,  10,  20,  20,   5,   5,   5,
    10,  10,  10,  20,  20,  10,  10,  10,
    20,  20,  20,  30,  30,  30,  20,  20,
    30,  30,  30,  40,  40,  30,  30,  30,
    90,  90,  90,  90,  90,  90,  90,  90
];

// knight positional score
#[rustfmt::skip]
const KNIGHT_SCORE: [i32; 64] =
[
    -5, -10,   0,   0,   0,   0, -10,  -5,
    -5,   0,   0,   0,   0,   0,   0,  -5,
    -5,   5,  20,  10,  10,  20,   5,  -5,
    -5,  10,  20,  30,  30,  20,  10,  -5,
    -5,  10,  20,  30,  30,  20,  10,  -5,
    -5,   5,  20,  20,  20,  20,   5,  -5,
    -5,   0,   0,  10,  10,   0,   0,  -5,
    -5,   0,   0,   0,   0,   0,   0,  -5
 ];

// bishop positional score
#[rustfmt::skip]
const BISHOP_SCORE: [i32; 64] =
[
     0,   0, -10,   0,   0, -10,   0,   0,
     0,  30,   0,   0,   0,   0,  30,   0,
     0,  10,   0,   0,   0,   0,  10,   0,
     0,   0,  10,  20,  20,  10,   0,   0,
     0,   0,  10,  20,  20,  10,   0,   0,
     0,   0,   0,  10,  10,   0,   0,   0,
     0,   0,   0,   0,   0,   0,   0,   0,
     0,   0,   0,   0,   0,   0,   0,   0
];

// rook positional score
#[rustfmt::skip]
const ROOK_SCORE: [i32; 64] =
[
     0,   0,   0,  20,  20,   0,   0,   0,
     0,   0,  10,  20,  20,  10,   0,   0,
     0,   0,  10,  20,  20,  10,   0,   0,
     0,   0,  10,  20,  20,  10,   0,   0,
     0,   0,  10,  20,  20,  10,   0,   0,
     0,   0,  10,  20,  20,  10,   0,   0,
    50,  50,  50,  50,  50,  50,  50,  50,
    50,  50,  50,  50,  50,  50,  50,  50

];

// king positional score
#[rustfmt::skip]
const KING_SCORE: [i32; 64] =
[
     0,   0,   5,   0, -15,   0,  10,   0,
     0,   5,   5,  -5,  -5,   0,   5,   0,
     0,   0,   5,  10,  10,   5,   0,   0,
     0,   5,  10,  20,  20,  10,   5,   0,
     0,   5,  10,  20,  20,  10,   5,   0,
     0,   5,   5,  10,  10,   5,   5,   0,
     0,   0,   5,   5,   5,   5,   0,   0,
     0,   0,   0,   0,   0,   0,   0,   0
];

const fn get_pawn_score(sq: Square, c: Color) -> i32 {
    match c {
        Color::White => PAWN_SCORE[sq as usize],
        Color::Black => PAWN_SCORE[sq.mirror() as usize],
    }
}

const fn get_knight_score(sq: Square, c: Color) -> i32 {
    match c {
        Color::White => KNIGHT_SCORE[sq as usize],
        Color::Black => KNIGHT_SCORE[sq.mirror() as usize],
    }
}

const fn get_bishop_score(sq: Square, c: Color) -> i32 {
    match c {
        Color::White => BISHOP_SCORE[sq as usize],
        Color::Black => BISHOP_SCORE[sq.mirror() as usize],
    }
}

const fn get_rook_score(sq: Square, c: Color) -> i32 {
    match c {
        Color::White => ROOK_SCORE[sq as usize],
        Color::Black => ROOK_SCORE[sq.mirror() as usize],
    }
}

const fn get_king_score(sq: Square, c: Color) -> i32 {
    match c {
        Color::White => KING_SCORE[sq as usize],
        Color::Black => KING_SCORE[sq.mirror() as usize],
    }
}

pub fn evaluate(config: &BoardConfig) -> i32 {
    let mut score = 0;
    use BoardPiece::*;
    for p in BoardPiece::iter() {
        let mut bb = config.bitboards[p as usize];
        while *bb > 0 {
            let pos = bb.pop_sq().unwrap();
            let mat_score = MATERIAL_SCORE[p as usize];
            let pos_score = match p {
                WhiteKing | BlackKing => get_king_score(pos, p.get_color()),
                WhitePawn | BlackPawn => get_pawn_score(pos, p.get_color()),
                WhiteRook | BlackRook => get_rook_score(pos, p.get_color()),
                // WhiteQueen | BlackQueen => get_queen_score(pos, p.get_color()),
                WhiteQueen | BlackQueen => 0,
                WhiteBishop | BlackBishop => get_bishop_score(pos, p.get_color()),
                WhiteKnight | BlackKnight => get_knight_score(pos, p.get_color()),
            };

            score += match p.get_color() {
                Color::White => mat_score + pos_score,
                Color::Black => mat_score - pos_score,
            }
        }
    }
    score
}

#[rustfmt::skip]
const MVV_LVA: [[i32; 12]; 12] = [
    [105, 205, 305, 405, 505, 605,  105, 205, 305, 405, 505, 605],
    [104, 204, 304, 404, 504, 604,  104, 204, 304, 404, 504, 604],
    [103, 203, 303, 403, 503, 603,  103, 203, 303, 403, 503, 603],
    [102, 202, 302, 402, 502, 602,  102, 202, 302, 402, 502, 602],
    [101, 201, 301, 401, 501, 601,  101, 201, 301, 401, 501, 601],
    [100, 200, 300, 400, 500, 600,  100, 200, 300, 400, 500, 600],

    [105, 205, 305, 405, 505, 605,  105, 205, 305, 405, 505, 605],
    [104, 204, 304, 404, 504, 604,  104, 204, 304, 404, 504, 604],
    [103, 203, 303, 403, 503, 603,  103, 203, 303, 403, 503, 603],
    [102, 202, 302, 402, 502, 602,  102, 202, 302, 402, 502, 602],
    [101, 201, 301, 401, 501, 601,  101, 201, 301, 401, 501, 601],
    [100, 200, 300, 400, 500, 600,  100, 200, 300, 400, 500, 600],
];

pub fn score_mvv_lva(m: &Move) -> i32 {
    if m.capture.is_none() {
        return 0;
    }
    let atk = m.p;
    let victim = m.capture.unwrap();

    MVV_LVA[atk as usize][victim as usize]
}
