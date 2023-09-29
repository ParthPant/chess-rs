use rand::prelude::*;
use std::cell::RefCell;

thread_local! {
    pub static PRNG: RefCell<ThreadRng> = RefCell::new(thread_rng());
}

pub fn random_u64() -> u64 {
    PRNG.with(|rng| rng.borrow_mut().gen())
}