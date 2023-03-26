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
    quiescence_depth: usize,
    stats: AIStat,
    killer_moves: [[Option<Move>; Self::MAX_DEPTH]; 2],
    history_moves: [[i32; 64]; 12],
    table: TT,
    pv_length: [usize; 64],
    pv_table: [[Option<Move>; 64]; 64],
    score_pv: bool,
    follow_pv: bool,
}

impl Default for NegaMaxAI {
    fn default() -> Self {
        Self {
            depth: 5,
            quiescence_depth: 4,
            stats: Default::default(),
            killer_moves: [[None; Self::MAX_DEPTH]; 2],
            history_moves: [[0; Self::MAX_DEPTH]; 12],
            table: Default::default(),
            pv_length: [0; Self::MAX_DEPTH],
            pv_table: [[None; Self::MAX_DEPTH]; Self::MAX_DEPTH],
            score_pv: false,
            follow_pv: false,
        }
    }
}

impl NegaMaxAI {
    const MIN: i32 = -50000;
    const MAX: i32 = 50000;
    const MAX_DEPTH: usize = 64;

    fn score_move(&mut self, m: &Move, ply: usize) -> i32 {
        if self.score_pv && self.pv_table[0][ply] == Some(*m) {
            self.score_pv = false;
            self.follow_pv = true;
            return 20000;
        }
        if m.capture.is_some() {
            return score_mvv_lva(m);
        } else {
            if self.killer_moves[0][ply] == Some(*m) {
                return 9000;
            } else if self.killer_moves[1][ply] == Some(*m) {
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
        self.stats.node_count += 1;
        self.stats.max_depth = usize::max(self.stats.max_depth, depth);
        self.pv_length[ply] = ply;

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
            return self.quiescence(config, gen, alpha, beta, self.quiescence_depth, ply + 1);
        }
        if ply > Self::MAX_DEPTH - 1 {
            return evaluate(config);
        }

        let mut value = Self::MIN;
        let mut moves = gen.gen_all_moves(config.get_active_color(), &config, false);
        if self.follow_pv {
            if moves
                .data()
                .iter()
                .any(|m| self.pv_table[0][ply] == Some(*m))
            {
                self.score_pv = true;
                self.follow_pv = true;
            } else {
                self.follow_pv = false;
            }
        }
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

                if value >= beta {
                    if m.capture.is_none() {
                        self.killer_moves[1][ply] = self.killer_moves[0][ply];
                        self.killer_moves[0][ply] = Some(*m);
                    }
                    break;
                }

                if value > alpha {
                    alpha = value;

                    self.pv_table[ply][ply] = Some(*m);
                    for next_ply in (ply + 1)..self.pv_length[ply + 1] {
                        self.pv_table[ply][next_ply] = self.pv_table[ply + 1][next_ply];
                    }
                    self.pv_length[ply] = self.pv_length[ply + 1];

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
        depth: usize,
        ply: usize,
    ) -> i32 {
        self.stats.node_count += 1;

        let eval = evaluate(config);
        if depth == 0 {
            return eval;
        }
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
                let score = -self.quiescence(config, gen, -beta, -alpha, depth - 1, ply + 1);
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
        self.killer_moves = [[None; Self::MAX_DEPTH]; 2];
        self.pv_length = [0; 64];
        self.pv_table = [[None; 64]; 64];
        self.score_pv = false;
        self.follow_pv = false;

        let mut config = config.clone();
        let now = Instant::now();

        for current_depth in 1..(self.depth + 1) {
            self.follow_pv = true;
            self.stats.node_count = 0;
            self.nega_max(&mut config, gen, Self::MIN, Self::MAX, current_depth, 0);
        }
        // self.nega_max(&mut config, gen, Self::MIN, Self::MAX, self.depth, 0);

        self.stats.time = now.elapsed();
        self.pv_table[0][0]
    }

    fn get_stats(&self) -> AIStat {
        self.stats
    }
}
