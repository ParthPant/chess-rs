use crate::data::{BitBoard, BoardPiece, Color};
use rand;

pub const NOT_A_FILE: u64 = {
    let mut x: u64 = 0;
    let mut i = 0;
    while i < 64 {
        if i % 8 != 0 {
            x |= 1 << i;
        }
        i = i + 1;
    }
    x
};

pub const NOT_AB_FILE: u64 = {
    let mut x: u64 = 0;
    let mut i = 0;
    while i < 64 {
        if i % 8 != 0 && i % 8 != 1 {
            x |= 1 << i;
        }
        i = i + 1;
    }
    x
};

pub const NOT_H_FILE: u64 = {
    let mut x: u64 = 0;
    let mut i = 0;
    while i < 64 {
        if i % 8 != 7 {
            x |= 1 << i;
        }
        i = i + 1;
    }
    x
};

pub const NOT_GH_FILE: u64 = {
    let mut x: u64 = 0;
    let mut i = 0;
    while i < 64 {
        if i % 8 != 7 && i % 8 != 6 {
            x |= 1 << i;
        }
        i = i + 1;
    }
    x
};

pub const WP_ATK_TBL: [u64; 64] = {
    let mut i = 0;
    let mut table: [u64; 64] = [0; 64];
    while i < 64 {
        table[i] = generate_pawn_attack(Color::White, i);
        i += 1;
    }
    table
};

pub const BP_ATK_TBL: [u64; 64] = {
    let mut i = 0;
    let mut table: [u64; 64] = [0; 64];
    while i < 64 {
        table[i] = generate_pawn_attack(Color::Black, i);
        i += 1;
    }
    table
};

pub const N_ATK_TBL: [u64; 64] = {
    let mut i = 0;
    let mut table: [u64; 64] = [0; 64];
    while i < 64 {
        table[i] = generate_knight_attack(i);
        i += 1;
    }
    table
};

pub const K_ATK_TBL: [u64; 64] = {
    let mut i = 0;
    let mut table: [u64; 64] = [0; 64];
    while i < 64 {
        table[i] = generate_king_attack(i);
        i += 1;
    }
    table
};

const fn generate_pawn_attack(side: Color, sq: usize) -> u64 {
    let b = 1 << sq;
    match side {
        Color::White => ((b << 7) & NOT_H_FILE) | ((b << 9) & NOT_A_FILE),
        Color::Black => ((b >> 7) & NOT_A_FILE) | ((b >> 9) & NOT_H_FILE),
    }
}

const fn generate_knight_attack(sq: usize) -> u64 {
    let mut b = 1 << sq;
    // << up, >> down
    b = (b << 6) | (b << 15) | (b << 10) | (b << 17) | (b >> 6) | (b >> 15) | (b >> 10) | (b >> 17);

    if sq % 8 == 0 || sq % 8 == 1 {
        b &= NOT_GH_FILE;
    }

    if sq % 8 == 7 || sq % 8 == 6 {
        b &= NOT_AB_FILE;
    }

    b
}

const fn generate_king_attack(sq: usize) -> u64 {
    let mut b = 1 << sq;
    // << up, >> down
    b = (b << 8) | (b >> 8) | (b >> 1) | (b << 1) | (b >> 9) | (b << 9) | (b >> 7) | (b << 7);

    if sq % 8 == 0 {
        b &= NOT_H_FILE;
    }

    if sq % 8 == 7 {
        b &= NOT_A_FILE;
    }

    b
}

#[derive(Debug, Default, Clone, Copy)]
pub struct MagicEntry {
    pub relevant_occupancy: u64,
    pub magic: u64,
    pub index_bits: u8,
}

pub struct TableFillError;

pub fn magic_index(entry: &MagicEntry, blockers: BitBoard) -> usize {
    let blockers: u64 = blockers.into();
    let relevant_blockers = blockers & entry.relevant_occupancy;
    let hash = relevant_blockers.wrapping_mul(entry.magic);
    let index = (hash >> (64 - entry.index_bits)) as usize;
    index
}

fn try_make_table(
    sq: usize,
    slider: BoardPiece,
    magic: &MagicEntry,
) -> Result<Vec<BitBoard>, TableFillError> {
    let mut table = vec![BitBoard::default(); 1 << magic.index_bits];

    let mut subset: u64 = 0;
    loop {
        let table_entry = &mut table[magic_index(magic, subset.into())];
        let moves = match slider {
            BoardPiece::WhiteRook | BoardPiece::BlackRook => generate_rook_attack(sq, subset),
            BoardPiece::WhiteBishop | BoardPiece::BlackBishop => generate_bishop_attack(sq, subset),
            _ => panic!("{:?} is not a sliding Piece", slider),
        };

        if *table_entry == 0 {
            *table_entry = moves.into();
        } else if *table_entry != moves {
            return Err(TableFillError);
        }

        subset = subset.wrapping_sub(magic.relevant_occupancy) & magic.relevant_occupancy;
        if subset == 0 {
            break;
        }
    }

    Ok(table)
}

pub fn find_magic(sq: usize, slider: BoardPiece) -> (MagicEntry, Vec<BitBoard>) {
    let relevant_occupancy = match slider {
        BoardPiece::WhiteRook | BoardPiece::BlackRook => rook_relevant_occupancy(sq),
        BoardPiece::WhiteBishop | BoardPiece::BlackBishop => bishop_relevant_occupancy(sq),
        _ => panic!("{:?} is not a sliding Piece", slider),
    };

    loop {
        let magic = random_u64() & random_u64() & random_u64();
        let index_bits = relevant_occupancy.count_ones() as u8;
        let magic_entry = MagicEntry {
            relevant_occupancy,
            magic,
            index_bits,
        };
        if let Ok(table) = try_make_table(sq, slider, &magic_entry) {
            return (magic_entry, table);
        }
    }
}

fn random_u64() -> u64 {
    rand::random::<u64>()
}

const fn bishop_relevant_occupancy(sq: usize) -> u64 {
    let sq = sq as i8;
    let mut b = 0;

    let mut r = sq / 8 + 1;
    let mut f = sq % 8 + 1;
    while r <= 6 && f <= 6 {
        b |= 1 << (r * 8 + f);
        r += 1;
        f += 1;
    }

    let mut r = sq / 8 + 1;
    let mut f = sq % 8 - 1;
    while r <= 6 && f > 0 {
        b |= 1 << (r * 8 + f);
        r += 1;
        f -= 1;
    }

    let mut r = sq / 8 - 1;
    let mut f = sq % 8 + 1;
    while r > 0 && f <= 6 {
        b |= 1 << (r * 8 + f);
        r -= 1;
        f += 1;
    }

    let mut r = sq / 8 - 1;
    let mut f = sq % 8 - 1;
    while r > 0 && f > 0 {
        b |= 1 << (r * 8 + f);
        r -= 1;
        f -= 1;
    }

    b
}

const fn rook_relevant_occupancy(sq: usize) -> u64 {
    let sq = sq as i8;
    let mut b = 0;

    let mut r = sq / 8 + 1;
    let f = sq % 8;
    while r <= 6 {
        b |= 1 << (r * 8 + f);
        r += 1;
    }

    let mut r = sq / 8 - 1;
    let f = sq % 8;
    while r > 0 {
        b |= 1 << (r * 8 + f);
        r -= 1;
    }

    let r = sq / 8;
    let mut f = sq % 8 + 1;
    while f <= 6 {
        b |= 1 << (r * 8 + f);
        f += 1;
    }

    let r = sq / 8;
    let mut f = sq % 8 - 1;
    while f > 0 {
        b |= 1 << (r * 8 + f);
        f -= 1;
    }

    b
}

const fn generate_bishop_attack(sq: usize, block: u64) -> u64 {
    let sq = sq as i8;
    let mut b = 0;

    let mut r = sq / 8 + 1;
    let mut f = sq % 8 + 1;
    while r <= 7 && f <= 7 {
        b |= 1 << (r * 8 + f);
        if block & (1 << (r * 8 + f)) > 0 {
            break;
        }
        r += 1;
        f += 1;
    }

    let mut r = sq / 8 + 1;
    let mut f = sq % 8 - 1;
    while r <= 7 && f >= 0 {
        b |= 1 << (r * 8 + f);
        if block & (1 << (r * 8 + f)) > 0 {
            break;
        }
        r += 1;
        f -= 1;
    }

    let mut r = sq / 8 - 1;
    let mut f = sq % 8 + 1;
    while r >= 0 && f <= 7 {
        b |= 1 << (r * 8 + f);
        if block & (1 << (r * 8 + f)) > 0 {
            break;
        }
        r -= 1;
        f += 1;
    }

    let mut r = sq / 8 - 1;
    let mut f = sq % 8 - 1;
    while r >= 0 && f >= 0 {
        b |= 1 << (r * 8 + f);
        if block & (1 << (r * 8 + f)) > 0 {
            break;
        }
        r -= 1;
        f -= 1;
    }

    b
}

const fn generate_rook_attack(sq: usize, block: u64) -> u64 {
    let sq = sq as i8;
    let mut b = 0;

    let mut r = sq / 8 + 1;
    let f = sq % 8;
    while r <= 7 {
        b |= 1 << (r * 8 + f);
        if block & (1 << (r * 8 + f)) > 0 {
            break;
        }
        r += 1;
    }

    let mut r = sq / 8 - 1;
    let f = sq % 8;
    while r >= 0 {
        b |= 1 << (r * 8 + f);
        if block & (1 << (r * 8 + f)) > 0 {
            break;
        }
        r -= 1;
    }

    let r = sq / 8;
    let mut f = sq % 8 + 1;
    while f <= 7 {
        b |= 1 << (r * 8 + f);
        if block & (1 << (r * 8 + f)) > 0 {
            break;
        }
        f += 1;
    }

    let r = sq / 8;
    let mut f = sq % 8 - 1;
    while f >= 0 {
        b |= 1 << (r * 8 + f);
        if block & (1 << (r * 8 + f)) > 0 {
            break;
        }
        f -= 1;
    }

    b
}
