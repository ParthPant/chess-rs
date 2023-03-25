use std::collections::HashMap;

use crate::data::Move;

#[derive(Debug, Default)]
pub enum SearchFlag {
    #[default]
    Exact,
    Lowerbound,
    Upperbound,
}

#[derive(Debug, Default)]
pub struct TTEntry {
    pub key: u64,
    pub depth: usize,
    pub flag: SearchFlag,
    pub best: Option<Move>,
    pub value: i32,
}

pub type TT = HashMap<u64, TTEntry>;
