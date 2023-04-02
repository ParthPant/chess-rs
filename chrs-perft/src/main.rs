#![allow(warnings, unused)]

use chrs_lib::data::{BoardConfig, BoardPiece, Color, Move, Square};
use chrs_lib::generator::MoveGenerator;
use chrs_lib::zobrist::hash;
use std::env;
use std::str::FromStr;
use std::time::Instant;

fn perft_impl(depth: usize, config: &mut BoardConfig, gen: &MoveGenerator, divide: bool) -> usize {
    if depth == 0 {
        return 1;
    }

    let side = config.get_active_color();
    let moves = gen.gen_all_moves(side, config, false);

    if depth == 1 {
        return moves.len();
    }

    let mut count = 0;
    for m in moves.iter() {
        let key_scratch = hash(config);
        if let Some(commit) = config.make_move(*m) {
            let c = perft_impl(depth - 1, config, gen, false);
            if divide {
                println!("{} {}", commit.m.to_string().to_lowercase(), c);
            }
            count += c;
            config.undo_commit(&commit);
            let key_updated = config.get_hash();
            assert_eq!(key_scratch, key_updated);
        }
    }

    count
}

fn main() {
    let depth = std::env::args()
        .nth(1)
        .expect("Depth not provided")
        .parse()
        .unwrap();
    let fen = std::env::args().nth(2).expect("Fen not provided");
    let moves = std::env::args().nth(3).unwrap_or_default();

    let mut config = BoardConfig::from_fen_str(&fen);
    let gen = MoveGenerator::default();

    if moves != "" {
        for i in moves.split(' ').collect::<Vec<&str>>() {
            let chars = i.chars();
            let from: Square =
                Square::from_str(&chars.clone().take(2).collect::<String>()).unwrap();
            let to: Square =
                Square::from_str(&chars.clone().skip(2).take(2).collect::<String>()).unwrap();
            let mut m = Move::infer(from, to, &config);
            if m.is_empty_prom() {
                let mut p = format!("{}", chars.clone().last().unwrap());
                if config.get_active_color() == Color::White {
                    p = p.to_uppercase();
                }
                let prom = BoardPiece::from_str(&p).unwrap();
                m.set_prom(prom);
            }

            config.make_move(m);
        }
    }

    let now = Instant::now();
    let c = perft_impl(depth, &mut config, &gen, true);
    let elapsed = now.elapsed();
    println!("\n{}", c);
    println!("\nTime Take: {:?}", elapsed);
}
