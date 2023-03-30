mod eval;
mod negamax;
mod transposition;

use crate::data::{BoardConfig, Move};
use crate::generator::MoveGenerator;
pub use negamax::NegaMaxAI;
use std::time::Duration;

pub trait AI {
    fn get_best_move(&mut self, config: &BoardConfig, gen: &MoveGenerator) -> Option<Move>;
    fn get_stats(&self) -> AIStat;
}

#[derive(Default, Copy, Clone, Debug)]
pub struct AIStat {
    pub node_count: usize,
    pub time: Duration,
    pub max_depth: usize,
}
