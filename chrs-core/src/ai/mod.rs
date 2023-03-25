use std::time::{Duration, Instant};

use crate::{
    data::{BoardConfig, Move},
    generator::MoveGenerator,
};

use self::eval::*;

mod eval;

const MIN: i32 = -1000000000;
const MAX: i32 = 1000000000;

pub trait AI {
    fn get_best_move(&mut self, config: &BoardConfig, gen: &MoveGenerator) -> Option<Move>;
    fn get_stats(&self) -> AIStat;
}

#[derive(Default, Copy, Clone, Debug)]
pub struct AIStat {
    node_count: usize,
    time: Duration,
}

pub struct NegaMaxAI {
    depth: usize,
    stats: AIStat,
}

impl Default for NegaMaxAI {
    fn default() -> Self {
        Self {
            depth: 3,
            stats: Default::default(),
        }
    }
}

impl NegaMaxAI {
    fn nega_max(
        &mut self,
        config: &mut BoardConfig,
        gen: &MoveGenerator,
        mut alpha: i32,
        beta: i32,
        depth: usize,
    ) -> i32 {
        if depth == 0 {
            // return evaluate(config);
            return self.quiescence(config, gen, alpha, beta);
        }

        self.stats.node_count += 1;

        let mut value = MIN;
        let mut moves = gen.gen_all_moves(config.get_active_color(), &config, false);
        moves
            .data()
            .sort_by(|a, b| score_move(b, config, gen).cmp(&score_move(a, config, gen)));

        for m in moves.iter() {
            if let Some(commit) = config.make_move(*m) {
                value = i32::max(value, -self.nega_max(config, gen, -beta, -alpha, depth - 1));
                config.undo_commit(&commit);
                alpha = i32::max(alpha, value);
                if alpha >= beta {
                    break;
                }
            }
        }

        value
    }

    fn quiescence(
        &mut self,
        config: &mut BoardConfig,
        gen: &MoveGenerator,
        mut alpha: i32,
        beta: i32,
    ) -> i32 {
        self.stats.node_count += 1;

        let eval = evaluate(config);
        if eval >= beta {
            return beta;
        }
        if alpha < eval {
            alpha = eval;
        }

        let mut moves = gen.gen_all_moves(config.get_active_color(), &config, true);
        moves
            .data()
            .sort_by(|a, b| score_move(b, config, gen).cmp(&score_move(a, config, gen)));

        for m in moves.iter() {
            assert!(m.capture.is_some());
            if let Some(commit) = config.make_move(*m) {
                let score = -self.quiescence(config, gen, -beta, -alpha);
                config.undo_commit(&commit);
                if score >= beta {
                    return beta;
                }
                alpha = i32::max(alpha, score);
            }
        }
        alpha
    }
}

impl AI for NegaMaxAI {
    fn get_best_move(&mut self, config: &BoardConfig, gen: &MoveGenerator) -> Option<Move> {
        self.stats = Default::default();
        let mut best = None;
        let mut best_score = MIN;
        let mut config = config.clone();
        let moves = gen.gen_all_moves(config.get_active_color(), &config, false);
        let now = Instant::now();
        for m in moves.iter() {
            if let Some(commit) = config.make_move(*m) {
                let score = -self.nega_max(&mut config, gen, MIN, MAX, self.depth - 1);
                if score >= best_score {
                    best_score = score;
                    best = Some(*m);
                }
                config.undo_commit(&commit);
            }
        }
        self.stats.time = now.elapsed();
        best
    }

    fn get_stats(&self) -> AIStat {
        self.stats
    }
}
