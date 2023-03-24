use crate::{
    data::{BoardConfig, Move},
    generator::MoveGenerator,
};

use self::eval::evaluate;

mod eval;

const MIN: i32 = -1000000000;
const MAX: i32 = 1000000000;

pub trait AI {
    fn get_best_move(&self, config: &BoardConfig, gen: &MoveGenerator) -> Option<Move>;
}

pub struct NegaMaxAI {
    depth: usize,
}

impl Default for NegaMaxAI {
    fn default() -> Self {
        Self { depth: 5 }
    }
}

impl NegaMaxAI {
    fn nega_max(
        &self,
        config: &mut BoardConfig,
        gen: &MoveGenerator,
        mut alpha: i32,
        beta: i32,
        depth: usize,
    ) -> i32 {
        if depth == 0 {
            return evaluate(config);
        }

        let mut value = MIN;
        let moves = gen.gen_all_moves(config.get_active_color(), config);
        for m in moves.iter() {
            if let Some(commit) = config.make_move(*m) {
                let v = -self.nega_max(config, gen, alpha, beta, depth - 1);
                value = i32::max(value, v);
                config.undo_commit(&commit);
                alpha = i32::max(alpha, value);
                if alpha >= beta {
                    // log::debug!("Cut-off");
                    break;
                }
            }
        }

        value
    }
}

impl AI for NegaMaxAI {
    fn get_best_move(&self, config: &BoardConfig, gen: &MoveGenerator) -> Option<Move> {
        let mut best = None;
        let mut best_score = MIN;
        let mut config = config.clone();
        let moves = gen.gen_all_moves(config.get_active_color(), &config);
        for m in moves.iter() {
            if let Some(commit) = config.make_move(*m) {
                let score = -self.nega_max(&mut config, gen, MIN, MAX, self.depth - 1);
                // log::debug!("Candidate score: {}", score);
                if score >= best_score {
                    best_score = score;
                    best = Some(*m);
                }
                config.undo_commit(&commit);
            }
        }

        // log::debug!("Chosen score: {}", best_score);
        best
    }
}
