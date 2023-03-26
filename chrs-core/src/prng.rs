use rand::prelude::*;
use rand_chacha::ChaCha20Rng;
use std::cell::RefCell;

thread_local! {
    pub static PRNG: RefCell<ChaCha20Rng> = RefCell::new(ChaCha20Rng::seed_from_u64(234239432));
}

pub fn random_u64() -> u64 {
    PRNG.with(|rng| rng.borrow_mut().gen())
}
