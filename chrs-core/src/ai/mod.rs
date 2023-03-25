use std::time::{Duration, Instant};

use crate::{
    data::{BoardConfig, Move},
    generator::MoveGenerator,
};

use self::eval::*;

mod eval;

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
    killer_moves: [[Move; Self::MAX_DEPTH]; 2],
    history_moves: [[i32; 64]; 12],
}

impl Default for NegaMaxAI {
    fn default() -> Self {
        Self {
            depth: 4,
            stats: Default::default(),
            killer_moves: [[Move::default(); Self::MAX_DEPTH]; 2],
            history_moves: [[0; 64]; 12],
        }
    }
}

impl NegaMaxAI {
    const MIN: i32 = -50000;
    const MAX: i32 = 50000;
    const MAX_DEPTH: usize = 64;

    fn score_move(&self, m: &Move, config: &BoardConfig, ply: usize) -> i32 {
        if m.capture.is_some() {
            return score_mvv_lva(m, config);
        } else {
            if self.killer_moves[0][ply] == *m {
                return 9000;
            } else if self.killer_moves[1][ply] == *m {
                return 8000;
            } else {
                let p = config.get_at_sq(m.from).unwrap();
                return self.history_moves[p as usize][m.to as usize];
            }
        }
    }

    fn nega_max(
        &mut self,
        config: &mut BoardConfig,
        gen: &MoveGenerator,
        mut alpha: i32,
        beta: i32,
        depth: usize,
        ply: usize,
    ) -> i32 {
        if depth == 0 {
            // return evaluate(config);
            return self.quiescence(config, gen, alpha, beta, ply + 1);
        }
        if ply > Self::MAX_DEPTH - 1 {
            return evaluate(config);
        }

        self.stats.node_count += 1;

        let mut value = Self::MIN;
        let mut moves = gen.gen_all_moves(config.get_active_color(), &config, false);
        moves.data().sort_by(|a, b| {
            self.score_move(b, config, ply)
                .cmp(&self.score_move(a, config, ply))
        });

        for m in moves.iter() {
            if let Some(commit) = config.make_move(*m) {
                value = i32::max(
                    value,
                    -self.nega_max(config, gen, -beta, -alpha, depth - 1, ply + 1),
                );
                config.undo_commit(&commit);
                if value > alpha {
                    alpha = value;
                    let p = config.get_at_sq(m.from).unwrap();
                    if m.capture.is_none() {
                        self.history_moves[p as usize][m.to as usize] += (depth * depth) as i32;
                    }
                }
                alpha = i32::max(alpha, value);
                if alpha >= beta {
                    self.killer_moves[1][ply] = self.killer_moves[0][ply];
                    self.killer_moves[0][ply] = *m;
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
        ply: usize,
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
        moves.data().sort_by(|a, b| {
            self.score_move(b, config, ply)
                .cmp(&self.score_move(a, config, ply))
        });

        for m in moves.iter() {
            assert!(m.capture.is_some());
            if let Some(commit) = config.make_move(*m) {
                let score = -self.quiescence(config, gen, -beta, -alpha, ply + 1);
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
        self.history_moves = [[0; 64]; 12];
        self.killer_moves = [[Move::default(); Self::MAX_DEPTH]; 2];

        let mut best = None;
        let mut best_score = Self::MIN;
        let mut config = config.clone();
        let moves = gen.gen_all_moves(config.get_active_color(), &config, false);
        let now = Instant::now();
        for m in moves.iter() {
            if let Some(commit) = config.make_move(*m) {
                let score =
                    -self.nega_max(&mut config, gen, Self::MIN, Self::MAX, self.depth - 1, 1);
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
