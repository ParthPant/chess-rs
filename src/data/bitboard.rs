#[derive(Debug, Clone, Copy, Default)]
pub struct BitBoard(u64);

impl BitBoard {
    pub fn add(&mut self, x: usize, y: usize) {
        self.0 |= (1 as u64) << Self::bit_from_xy(x, y);
    }

    pub fn remove(&mut self, x: usize, y: usize) {
        self.0 &= !((1 as u64) << Self::bit_from_xy(x, y) as u64);
    }

    pub fn move_xy_to_xy(&mut self, prev: (usize, usize), new: (usize, usize)) {
        self.remove(prev.0, prev.1);
        self.add(new.0, new.1);
    }

    pub fn data(&self) -> u64 {
        self.0
    }

    fn bit_from_xy(x: usize, y: usize) -> u64 {
        (y * 8 + x) as u64
    }
}
