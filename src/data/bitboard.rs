use std::fmt::Display;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign};

#[derive(Debug, Clone, Copy, Default, PartialOrd, Ord, Eq, PartialEq)]
pub struct BitBoard(u64);

impl BitBoard {
    pub fn add(&mut self, x: usize, y: usize) {
        self.0 |= (1 as u64) << Self::bit_from_xy(x, y);
    }

    pub fn remove(&mut self, x: usize, y: usize) {
        self.0 &= !((1 as u64) << Self::bit_from_xy(x, y) as u64);
    }

    pub fn set(&mut self, sq: usize) {
        self.0 |= (1 as u64) << sq;
    }

    pub fn unset(&mut self, sq: usize) {
        self.0 &= !((1 as u64) << sq);
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

    fn xy_from_bit(i: usize) -> (usize, usize) {
        (i % 8, i / 8)
    }
}

impl From<BitBoard> for u64 {
    fn from(value: BitBoard) -> Self {
        value.0
    }
}

impl From<u64> for BitBoard {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl Display for BitBoard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut b = (0..8)
            .rev()
            .map(|x| {
                format!(
                    "{:08b} {}",
                    ((self.0 & (0xff << x * 8)) >> x * 8) as u8,
                    x + 1
                )
            })
            .map(|s| s.chars().rev().collect::<String>())
            .collect::<Vec<String>>()
            .join("\n");
        b.push_str("\n  abcdefgh");
        b = b.replace("0", ".");
        write!(f, "{}", b)
    }
}

impl BitOr for BitBoard {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for BitBoard {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = Self(self.0 | rhs.0)
    }
}

impl BitAnd for BitBoard {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl BitAndAssign for BitBoard {
    fn bitand_assign(&mut self, rhs: Self) {
        *self = Self(self.0 & rhs.0)
    }
}
