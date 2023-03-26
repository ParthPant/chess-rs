use super::eval::*;
use super::transposition::{SearchFlag, TT};
use super::{AIStat, AI};
use crate::{
    data::{BoardConfig, Move},
    generator::MoveGenerator,
};
use std::time::Instant;

pub struct NegaMaxAI {
    depth: usize,
    stats: AIStat,
    killer_moves: [[Move; Self::MAX_DEPTH]; 2],
    history_moves: [[i32; 64]; 12],
    table: TT,
}

impl Default for NegaMaxAI {
    fn default() -> Self {
        Self {
            depth: 3,
            stats: Default::default(),
            killer_moves: [[Move::default(); Self::MAX_DEPTH]; 2],
            history_moves: [[0; 64]; 12],
            table: Default::default(),
        }
    }
}

impl NegaMaxAI {
    const MIN: i32 = -50000;
    const MAX: i32 = 50000;
    const MAX_DEPTH: usize = 64;

    fn score_move(&self, m: &Move, ply: usize) -> i32 {
        if m.capture.is_some() {
            return score_mvv_lva(m);
        } else {
            if self.killer_moves[0][ply] == *m {
                return 9000;
            } else if self.killer_moves[1][ply] == *m {
                return 8000;
            } else {
                return self.history_moves[m.p as usize][m.to as usize];
            }
        }
    }

    fn nega_max(
        &mut self,
        config: &mut BoardConfig,
        gen: &MoveGenerator,
        mut alpha: i32,
        mut beta: i32,
        depth: usize,
        ply: usize,
    ) -> i32 {
        let alpha_orig = alpha;

        if let Some(entry) = self.table.get(&config.get_hash()) {
            if entry.depth >= depth {
                match entry.flag {
                    SearchFlag::Exact => return entry.value,
                    SearchFlag::Lowerbound => alpha = i32::max(alpha, entry.value),
                    SearchFlag::Upperbound => beta = i32::min(beta, entry.value),
                };
            }

            if alpha >= beta {
                return entry.value;
            }
        }

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
        moves
            .data()
            .sort_by(|a, b| self.score_move(b, ply).cmp(&self.score_move(a, ply)));

        for m in moves.iter() {
            if let Some(commit) = config.make_move(*m) {
                value = i32::max(
                    value,
                    -self.nega_max(config, gen, -beta, -alpha, depth - 1, ply + 1),
                );
                config.undo_commit(&commit);
                if alpha >= beta {
                    if m.capture.is_none() {
                        self.killer_moves[1][ply] = self.killer_moves[0][ply];
                        self.killer_moves[0][ply] = *m;
                    }
                    break;
                }
                if value > alpha {
                    alpha = value;
                    if m.capture.is_none() {
                        self.history_moves[m.p as usize][m.to as usize] += (depth * depth) as i32;
                    }
                }
            }
        }

        let entry = self
            .table
            .entry(config.get_hash())
            .or_insert(Default::default());
        entry.value = value;
        if value <= alpha_orig {
            entry.flag = SearchFlag::Upperbound;
        } else if value >= beta {
            entry.flag = SearchFlag::Lowerbound;
        } else {
            entry.flag = SearchFlag::Exact;
        }
        entry.depth = depth;

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
        alpha = i32::max(alpha, eval);

        let mut moves = gen.gen_all_moves(config.get_active_color(), &config, true);
        moves
            .data()
            .sort_by(|a, b| self.score_move(b, ply).cmp(&self.score_move(a, ply)));

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
